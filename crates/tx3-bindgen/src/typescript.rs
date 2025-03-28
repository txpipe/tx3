use convert_case::{Case, Casing};
use std::{collections::HashMap, rc::Rc};
use swc_common::{SyntaxContext, DUMMY_SP};
use swc_core::{self, atoms::Atom};
use swc_ecma_ast::*;

use super::Job;

fn ts_type_for_field(ty: &tx3_lang::ir::Type) -> TsType {
    match ty {
        tx3_lang::ir::Type::Int => TsType::TsKeywordType(TsKeywordType {
            span: DUMMY_SP,
            kind: TsKeywordTypeKind::TsNumberKeyword,
        }),
        tx3_lang::ir::Type::Address => TsType::TsKeywordType(TsKeywordType {
            span: DUMMY_SP,
            kind: TsKeywordTypeKind::TsStringKeyword,
        }),
        tx3_lang::ir::Type::Bool => TsType::TsKeywordType(TsKeywordType {
            span: DUMMY_SP,
            kind: TsKeywordTypeKind::TsBooleanKeyword,
        }),
        tx3_lang::ir::Type::Bytes => TsType::TsTypeRef(TsTypeRef {
            span: DUMMY_SP,
            type_name: TsEntityName::Ident(Ident::new_no_ctxt("Uint8Array".into(), DUMMY_SP)),
            type_params: None,
        }),
        tx3_lang::ir::Type::UtxoRef => TsType::TsKeywordType(TsKeywordType {
            span: DUMMY_SP,
            kind: TsKeywordTypeKind::TsStringKeyword,
        }),
        // Add other type mappings as needed
        _ => TsType::TsKeywordType(TsKeywordType {
            span: DUMMY_SP,
            kind: TsKeywordTypeKind::TsUnknownKeyword,
        }),
    }
}

fn map_into_object_lit(headers: &HashMap<String, String>) -> Expr {
    Expr::Object(ObjectLit {
        span: DUMMY_SP,
        props: headers
            .iter()
            .map(|(key, value)| {
                PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                    key: PropName::Str(Str::from(key.clone())),
                    value: Box::new(Expr::Lit(Lit::Str(Str::from(value.clone())))),
                })))
            })
            .collect(),
    })
}

pub fn generate(job: &Job) {
    let mut module_items = Vec::new();

    module_items.push(swc_ecma_quote::quote!(
        "import { TRPClient, TirEnvelope, TxEnvelope, ClientOptions } from 'tx3-trp';"
            as ModuleItem
    ));

    module_items.push(swc_ecma_quote::quote!(
        "export const DEFAULT_TRP_ENDPOINT = $value;" as ModuleItem,
        value: Expr = job.trp_endpoint.clone().into(),
    ));

    let headers_lit = map_into_object_lit(&job.trp_headers);

    let headers_item = swc_ecma_quote::quote!(
        "export const DEFAULT_HEADERS = $value;" as ModuleItem,
        value: Expr = headers_lit,
    );

    module_items.push(headers_item);

    let env_args_lit = map_into_object_lit(&job.env_args);

    let env_args_item = swc_ecma_quote::quote!(
        "export const DEFAULT_ENV_ARGS = $value;" as ModuleItem,
        value: Expr = env_args_lit,
    );

    module_items.push(env_args_item);

    let mut class_members = Vec::new();

    // Add private client field
    class_members.push(ClassMember::PrivateProp(PrivateProp {
        key: PrivateName {
            name: Atom::from("client"),
            ..Default::default()
        },
        type_ann: Some(Box::new(TsTypeAnn {
            span: DUMMY_SP,
            type_ann: Box::new(TsType::TsTypeRef(TsTypeRef {
                span: DUMMY_SP,
                type_name: TsEntityName::Ident(Ident::new_no_ctxt("TRPClient".into(), DUMMY_SP)),
                type_params: None,
            })),
        })),
        readonly: true,
        ..Default::default()
    }));

    // Add constructor
    class_members.push(ClassMember::Constructor(Constructor {
        span: DUMMY_SP,
        key: PropName::Ident(IdentName::new(Atom::from("constructor"), DUMMY_SP)),
        params: vec![ParamOrTsParamProp::Param(Param {
            span: DUMMY_SP,
            decorators: vec![],
            pat: Pat::Ident(BindingIdent {
                id: Ident::new_no_ctxt("options".into(), DUMMY_SP),
                type_ann: Some(Box::new(TsTypeAnn {
                    span: DUMMY_SP,
                    type_ann: Box::new(TsType::TsTypeRef(TsTypeRef {
                        span: DUMMY_SP,
                        type_name: TsEntityName::Ident(Ident::new_no_ctxt(
                            "ClientOptions".into(),
                            DUMMY_SP,
                        )),
                        type_params: None,
                    })),
                })),
            }),
        })],
        body: Some(BlockStmt {
            span: DUMMY_SP,
            stmts: vec![swc_ecma_quote::quote!(
                "this.#client = new TRPClient(options);" as Stmt
            )],
            ctxt: SyntaxContext::empty(),
        }),
        ..Default::default()
    }));

    for tx_def in job.protocol.txs() {
        let tx_name = tx_def.name.as_str();
        let prototx = job.protocol.new_tx(&tx_def.name).unwrap();

        let ir_bytes_hex = hex::encode(prototx.ir_bytes());

        let type_name = format!("{}Params", tx_name.to_case(Case::Pascal));
        let fn_name = format!("{}Tx", tx_name.to_case(Case::Camel));
        let ir_const_name = format!("{}_IR", tx_name.to_case(Case::Constant));

        let type_members = prototx
            .find_params()
            .iter()
            .map(|(key, type_)| {
                let field_name = key.as_str().to_case(Case::Camel);

                TsTypeElement::TsPropertySignature(TsPropertySignature {
                    span: DUMMY_SP,
                    readonly: false,
                    key: Box::new(Expr::Ident(Ident::new_no_ctxt(field_name.into(), DUMMY_SP))),
                    computed: false,
                    optional: false,
                    type_ann: Box::new(TsTypeAnn {
                        span: DUMMY_SP,
                        type_ann: ts_type_for_field(type_).into(),
                    })
                    .into(),
                })
            })
            .collect();

        module_items.push(ModuleItem::ModuleDecl(ModuleDecl::ExportDecl(ExportDecl {
            decl: Decl::TsTypeAlias(Box::new(TsTypeAliasDecl {
                span: DUMMY_SP,
                declare: false,
                id: Ident::new_no_ctxt(type_name.clone().into(), DUMMY_SP),
                type_params: None,
                type_ann: Box::new(TsType::TsTypeLit(TsTypeLit {
                    span: DUMMY_SP,
                    members: type_members,
                })),
            })),
            span: DUMMY_SP,
        })));

        let ir_define_stmt = swc_ecma_quote::quote!(
            "export const $ir_const_name = { bytecode: $ir_bytes, encoding: 'hex', version: 'v1alpha1' };" as ModuleItem,
            ir_const_name =  Ident::new_no_ctxt(ir_const_name.clone().into(), DUMMY_SP),
            ir_bytes: Expr = ir_bytes_hex.into(),
        );

        module_items.push(ir_define_stmt);

        // Add method to class
        let method = ClassMethod {
            kind: MethodKind::Method,
            span: DUMMY_SP,
            key: PropName::Ident(IdentName::new(Atom::from(fn_name), DUMMY_SP)),
            function: Box::new(Function {
                params: vec![Param {
                    span: DUMMY_SP,
                    decorators: vec![],
                    pat: Pat::Ident(BindingIdent {
                        id: Ident::new_no_ctxt("args".into(), DUMMY_SP),
                        type_ann: Some(Box::new(TsTypeAnn {
                            span: DUMMY_SP,
                            type_ann: Box::new(TsType::TsTypeRef(TsTypeRef {
                                span: DUMMY_SP,
                                type_name: TsEntityName::Ident(Ident::new_no_ctxt(
                                    type_name.into(),
                                    DUMMY_SP,
                                )),
                                type_params: None,
                            })),
                        })),
                    }),
                }],
                decorators: vec![],
                span: DUMMY_SP,
                body: Some(BlockStmt {
                    span: DUMMY_SP,
                    stmts: vec![swc_ecma_quote::quote!(
                        "return await this.#client.resolve({
                            tir: $ir_const_name,
                            args,
                        });" as Stmt,
                        ir_const_name = Ident::new_no_ctxt(ir_const_name.clone().into(), DUMMY_SP),
                    )],
                    ctxt: SyntaxContext::empty(),
                }),
                is_generator: false,
                is_async: true,
                type_params: None,
                return_type: Some(Box::new(TsTypeAnn {
                    span: DUMMY_SP,
                    type_ann: Box::new(TsType::TsTypeRef(TsTypeRef {
                        span: DUMMY_SP,
                        type_name: TsEntityName::Ident(Ident::new_no_ctxt(
                            "Promise<TxEnvelope>".into(),
                            DUMMY_SP,
                        )),
                        type_params: None,
                    })),
                })),
                ctxt: SyntaxContext::empty(),
            }),
            ..Default::default()
        };

        class_members.push(ClassMember::Method(method));
    }

    // Create and export the Client class
    module_items.push(ModuleItem::ModuleDecl(ModuleDecl::ExportDecl(ExportDecl {
        span: DUMMY_SP,
        decl: Decl::Class(ClassDecl {
            ident: Ident::new_no_ctxt("Client".into(), DUMMY_SP),
            declare: false,
            class: Box::new(Class {
                body: class_members,
                ..Default::default()
            }),
        }),
    })));

    module_items.push(swc_ecma_quote::quote!(
        "export const protocol = new Client({
            endpoint: DEFAULT_TRP_ENDPOINT,
            headers: DEFAULT_HEADERS,
            envArgs: DEFAULT_ENV_ARGS,
        });" as ModuleItem
    ));

    let module = Module {
        span: DUMMY_SP,
        body: module_items,
        shebang: None,
    };

    // Use swc_ecma_codegen to generate the final TypeScript code
    let mut buf = vec![];

    let writer = swc_ecma_codegen::text_writer::JsWriter::new(
        Rc::new(swc_common::SourceMap::default()),
        "\n",
        &mut buf,
        None,
    );

    let config = swc_ecma_codegen::Config::default();

    let mut emitter = swc_ecma_codegen::Emitter {
        cfg: config,
        comments: None,
        cm: Rc::new(swc_common::SourceMap::default()),
        wr: writer,
    };

    emitter.emit_module(&module).unwrap();

    std::fs::write(job.dest_path.join(format!("{}.ts", job.name)), buf)
        .expect("Failed to write TypeScript output");
}
