use jsc::JSCValueExtManual;

use crate::common::{Language, Session, SessionManager, User};

#[allow(dead_code)]
pub trait ToJSCValue {
    fn to_jscvalue(&self, context: &jsc::Context) -> jsc::Value;
}

impl ToJSCValue for Language {
    fn to_jscvalue(&self, context: &jsc::Context) -> jsc::Value {
        let value = jsc::Value::new_object(context, None, None);

        let code = self.code();
        let name = self.name();
        let territory = self.territory();

        value.object_set_property("code", &jsc::Value::new_string(context, Some(code)));
        value.object_set_property("name", &jsc::Value::new_string(context, Some(name)));
        value.object_set_property(
            "territory",
            &jsc::Value::new_string(context, Some(territory)),
        );

        value
    }
}

impl ToJSCValue for Session {
    fn to_jscvalue(&self, context: &jsc::Context) -> jsc::Value {
        let value = jsc::Value::new_object(context, None, None);

        let comment = self.comment();
        let key = self.key();
        let name = self.name();
        let session_type = self.type_();

        value.object_set_property("comment", &jsc::Value::new_string(context, Some(comment)));
        value.object_set_property("key", &jsc::Value::new_string(context, Some(key)));
        value.object_set_property("name", &jsc::Value::new_string(context, Some(name)));
        value.object_set_property("type", &jsc::Value::new_string(context, Some(session_type)));

        value
    }
}

impl ToJSCValue for User {
    fn to_jscvalue(&self, context: &jsc::Context) -> jsc::Value {
        let value = jsc::Value::new_object(context, None, None);

        let username = self.user_name();
        let real_name = self.real_name().filter(|n| !n.is_empty());
        let display_name = if real_name.is_some() {
            real_name
        } else {
            self.user_name()
        };
        let home_directory = self.home_directory();
        let image = self.icon_file();
        let language = self.language();
        let logged_in = self
            .uid()
            .map(|uid| SessionManager::is_logged_in(uid as u32))
            .unwrap_or_default();
        let session = self.session();

        value.object_set_property(
            "display_name",
            &jsc::Value::new_string(context, display_name),
        );
        value.object_set_property(
            "home_directory",
            &jsc::Value::new_string(context, home_directory),
        );
        value.object_set_property("image", &jsc::Value::new_string(context, image));
        value.object_set_property("language", &jsc::Value::new_string(context, language));
        value.object_set_property("logged_in", &jsc::Value::new_boolean(context, logged_in));
        value.object_set_property("session", &jsc::Value::new_string(context, session));
        value.object_set_property("username", &jsc::Value::new_string(context, username));

        value
    }
}
