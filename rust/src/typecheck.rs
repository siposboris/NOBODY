//! Conservative Rust type checking for canonical Nobody IR.
//! Unknown expressions are deferred; this is NOT a sound whole-program type system.
use std::collections::{BTreeMap,BTreeSet};
use crate::{lower_source,Diagnostic};
use crate::ir::{Module,IrStatement,IrExpr,Name};

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Type { Int, Float, Str, Bool, Null, Unknown }
impl Type {
    fn from_annotation(name:&str)->Self {
        match name {"int"=>Self::Int,"float"=>Self::Float,"str"=>Self::Str,
            "bool"=>Self::Bool,_=>Self::Unknown}
    }
}
fn diagnostic(code:u16,detail:String,repair:&'static str)->Diagnostic {
    // IR v1 does not carry statement spans; do not fabricate line numbers.
    Diagnostic{code,line:0,column:0,detail,repair}
}
fn compatible(expected:Type,actual:Type)->bool {
    expected==Type::Unknown || actual==Type::Unknown || expected==actual ||
        (expected==Type::Float && actual==Type::Int)
}
fn infer(expr:&IrExpr,env:&BTreeMap<String,Type>)->Type {
    match expr {
        IrExpr::Number(n)=>if n.contains('.') || n.contains('e') || n.contains('E') {Type::Float}else{Type::Int},
        IrExpr::String(_)=>Type::Str,
        IrExpr::Boolean(_)=>Type::Bool,
        IrExpr::Null=>Type::Null,
        IrExpr::Name(Name::User(name))=>*env.get(name).unwrap_or(&Type::Unknown),
        IrExpr::Name(Name::Builtin(_))|IrExpr::Call{..}=>Type::Unknown,
        IrExpr::Unary{op,value}=>{
            let t=infer(value,env);
            if op=="not" {Type::Bool}
            else if (op=="-" || op=="+") && matches!(t,Type::Int|Type::Float){t}
            else{Type::Unknown}
        }
        IrExpr::Binary{op,lhs,rhs}=>{
            let a=infer(lhs,env);let b=infer(rhs,env);
            if matches!(op.as_str(),"=="|"!="|"<"|">"|"<="|">="|"and"|"or"){return Type::Bool;}
            if op=="+" && a==Type::Str && b==Type::Str{return Type::Str;}
            if matches!(op.as_str(),"+"|"-"|"*"|"/"|"//"|"%"|"**")
                && matches!(a,Type::Int|Type::Float)
                && matches!(b,Type::Int|Type::Float){
                if op=="/" || a==Type::Float || b==Type::Float {Type::Float}else{Type::Int}
            }else{Type::Unknown}
        }
    }
}
fn block(statements:&[IrStatement],env:&mut BTreeMap<String,Type>)->Result<(),Diagnostic>{
    let mut declared_here=BTreeSet::new();
    for stmt in statements {
        match stmt {
            IrStatement::Let{name,type_name,expr}=>{
                if !declared_here.insert(name.clone()){
                    return Err(diagnostic(1004,
                        format!("Duplicate declaration of {name:?} in the same lexical block"),
                        "Rename the second declaration, or use assignment without 'let' to update the existing variable."));
                }
                let found=infer(expr,env);
                let declared=type_name.as_deref().map(Type::from_annotation).unwrap_or(found);
                if !compatible(declared,found){
                    return Err(diagnostic(1001,
                        format!("Variable {name:?} declares {declared:?}, but initializer has known type {found:?}"),
                        "Change the initializer to the declared type or correct the type annotation after ':'."));
                }
                env.insert(name.clone(),declared);
            }
            IrStatement::Assign{name,expr}=>{
                if let Some(expected)=env.get(name).copied(){
                    let actual=infer(expr,env);
                    if !compatible(expected,actual){
                        return Err(diagnostic(1001,
                            format!("Assignment to {name:?} expects {expected:?}, but expression has known type {actual:?}"),
                            "Assign a value compatible with the original declaration, or change that declaration."));
                    }
                }
                else {
                    return Err(diagnostic(1003,
                        format!("Assignment targets undeclared variable {name:?} in the visible lexical scope"),
                        "Declare the variable with 'let' before assigning to it, or correct the misspelled name."));
                }
            }
            IrStatement::If{branches,otherwise}=>{
                for (_,body) in branches{
                    let mut local=env.clone();block(body,&mut local)?;
                }
                if let Some(body)=otherwise{
                    let mut local=env.clone();block(body,&mut local)?;
                }
            }
            IrStatement::While{body,..}=>{
                let mut local=env.clone();block(body,&mut local)?;
            }
            IrStatement::For{variable,body,..}=>{
                let mut local=env.clone();local.insert(variable.clone(),Type::Unknown);
                block(body,&mut local)?;
            }
            IrStatement::Function{name,parameters,body}=>{
                if !declared_here.insert(name.clone()){
                    return Err(diagnostic(1004,
                        format!("Duplicate function/variable declaration {name:?} in the same lexical block"),
                        "Rename the function or remove the earlier declaration in this block."));
                }
                env.insert(name.clone(),Type::Unknown);
                let mut local=env.clone();
                let mut unique_parameters=BTreeSet::new();
                for param in parameters {
                    if !unique_parameters.insert(param.clone()){
                        return Err(diagnostic(1004,
                            format!("Function {name:?} repeats parameter {param:?}"),
                            "Give every parameter in the function signature a unique name."));
                    }
                    local.insert(param.clone(),Type::Unknown);
                }
                block(body,&mut local)?;
            }
            IrStatement::Return(_)|IrStatement::Break|IrStatement::Continue|
            IrStatement::Expression(_)=>{}
        }
    }
    Ok(())
}
pub fn check_ir(module:&Module)->Result<(),Diagnostic>{
    block(&module.statements,&mut BTreeMap::new())
}
pub fn check_source(source:&str,language:&str)->Result<(),Diagnostic>{
    check_ir(&lower_source(source,language)?)
}
#[cfg(test)]
mod tests{
    use super::*;
    #[test]fn rejects_known_wrong_initializer(){
        let e=check_source("let n: int = \"oops\"","ai").unwrap_err();
        assert_eq!(e.code,1001);assert!(e.detail.contains("n"));
    }
    #[test]fn rejects_known_wrong_assignment(){
        let e=check_source("let n: int = 1\nn = false","ai").unwrap_err();
        assert_eq!(e.code,1001);assert!(e.detail.contains("Assignment"));
    }
    #[test]fn accepts_compatible_numbers(){
        check_source("let x: float = 2\nx = 3.5","ai").unwrap();
    }
    #[test]fn rejects_boolean_as_integer(){
        assert_eq!(check_source("let x: int = true","ai").unwrap_err().code,1001);
    }
    #[test]fn accepts_unknown_function_results_conservatively(){
        check_source("let x: int = mystery(1)","ai").unwrap();
    }
    #[test]fn detects_wrong_type_in_nested_branch(){
        let e=check_source("let x: int = 1\nif true { x = \"bad\" }","ai").unwrap_err();
        assert_eq!(e.code,1001);
    }
    #[test]fn typed_shadowing_in_inner_block_does_not_leak(){
        check_source("let x: int = 1\nif true { let x: str = \"ok\" }\nx = 2","ai").unwrap();
    }
    #[test]fn rejects_undeclared_assignment(){
        let err=check_source("missing = 3","ai").unwrap_err();
        assert_eq!(err.code,1003);assert!(err.detail.contains("missing"));
    }
    #[test]fn rejects_duplicate_local_declaration(){
        let err=check_source("let x = 1\nlet x = 2","ai").unwrap_err();
        assert_eq!(err.code,1004);
    }
    #[test]fn allows_shadowing_in_child_block(){
        check_source("let x: int = 1\nif true { let x: str = \"ok\" }","ai").unwrap();
    }
    #[test]fn rejects_duplicate_function_parameter(){
        let err=check_source("fn f(x,x) { return x }","ai").unwrap_err();
        assert_eq!(err.code,1004);
    }
    #[test]fn rejects_function_and_variable_same_block(){
        let err=check_source("let f = 1\nfn f() { return 2 }","ai").unwrap_err();
        assert_eq!(err.code,1004);
    }
    #[test]fn accepts_assignment_to_visible_outer_variable(){
        check_source("let x: int = 1\nif true { x = 2 }","ai").unwrap();
    }
    #[test]fn checks_slovak_annotations(){
        assert_eq!(check_source("nech vek: cele = \"text\"","sk").unwrap_err().code,1001);
    }
}
