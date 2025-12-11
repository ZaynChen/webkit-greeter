// SPDX-FileCopyrightText: 2025 ZaynChen <zaynchen@qq.com>
//
// SPDX-License-Identifier: Apache-2.0

use jsc::{
    JSCValueExtManual,
    glib::{MainContext, VariantTy, clone, variant::ToVariant},
};
use wwpe::{ScriptWorld, UserMessage, WebPage};

pub fn web_page_initialize(api_script: String) {
    ScriptWorld::default()
        .expect("get default ScriptWorld failed")
        .connect_window_object_cleared(move |world, page, frame| {
            let context = frame.js_context_for_script_world(world).unwrap();
            let global_object = context.global_object().unwrap();

            global_object.object_set_property("send_request", &send_request(page, &context));
            context.evaluate(&api_script);

            page.connect_document_loaded(move |_| {
                global_object
                    .object_get_property("dispatch_ready_event")
                    .filter(|ready| ready.is_function())
                    .map(|ready| ready.function_callv(&[]));
            });

            page.connect_user_message_received(clone!(
                #[strong]
                context,
                move |_, message| user_message_received(message, &context)
            ));
        });
}

fn send_request(page: &WebPage, context: &jsc::Context) -> jsc::Value {
    jsc::Value::new_function_variadic(
        context,
        Some("send_request"),
        clone!(
            #[strong]
            page,
            #[strong]
            context,
            move |args| {
                if args.len() != 1 {
                    logger::warn!(
                        "Invalid number of arguments for send_request: len {}",
                        args.len()
                    );
                    return None;
                }
                let request = &args[0];
                if !request.object_has_property("target")
                    && !request.object_has_property("method")
                    && !request.object_has_property("args")
                {
                    logger::warn!("request is not a valid Request(target, method, args)");
                    return None;
                }

                let target = request.object_get_property("target").unwrap().to_str();
                let method = request.object_get_property("method").unwrap().to_str();
                let params = request
                    .object_get_property("args")
                    .unwrap()
                    .to_json(0)
                    .unwrap_or("[]".into());

                logger::debug!("{target}.{method}({params})");
                let message =
                    UserMessage::new(&target, Some(&[method.as_str(), &params].to_variant()));
                MainContext::default()
                    .block_on(page.send_message_to_view_future(&message))
                    .ok()
                    .map(|reply| {
                        if let Some(reply_params) = reply.parameters() {
                            jsc::Value::from_json(
                                &context,
                                reply_params
                                    .str()
                                    .expect("reply_params is not a json string"),
                            )
                        } else {
                            jsc::Value::new_undefined(&context)
                        }
                    })
            }
        ),
    )
}

fn user_message_received(message: &UserMessage, context: &jsc::Context) -> bool {
    if !matches!(
        message.name().as_deref(),
        Some("greeter") | Some("greeter_comm")
    ) {
        return false;
    }

    let msg_param = message.parameters().unwrap();
    if msg_param.is_type(VariantTy::ARRAY) {
        let p_len = msg_param.n_children();
        if p_len == 0 || p_len > 2 {
            return false;
        }
    } else {
        return false;
    }

    let name_var = msg_param.child_value(0);
    let params_var = msg_param.child_value(1);

    let name = name_var.str().unwrap();
    let json_params = params_var.str().unwrap();

    // logger::info!("{}.{name}({json_params})", message.name().unwrap());
    match message.name().as_deref() {
        Some("greeter") => {
            let _ = context
                .global_object()
                .unwrap()
                .object_get_property("greeter")
                .unwrap()
                .object_get_property(name)
                .unwrap_or_else(|| panic!("greeter does not has signal {name}"))
                .object_invoke_methodv("_emit", &[jsc::Value::from_json(context, json_params)]);

            true
        }
        Some("greeter_comm") => {
            if name != "_emit" {
                return false;
            }

            let data = jsc::Value::from_json(context, json_params)
                .object_get_property_at_index(0)
                .unwrap();

            let _ = context
                .global_object()
                .unwrap()
                .object_get_property("greeter_comm")
                .unwrap()
                .object_invoke_methodv("_emit", &[data]);

            true
        }
        _ => false,
    }
}
