use diesel::{self, prelude::*};

mod schema {
    table! {
        users {
            id -> Nullable<Integer>,
            email -> Text,
            first_name -> Text,
            last_name -> Text,
            access_token -> Text,
        }
    }
}

use self::schema::users;
use self::schema::users::dsl::{users as all_users};

#[table_name="users"]
#[derive(Serialize, Queryable, Insertable, Debug, Clone)]
pub struct User {
    pub id: Option<i32>,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub access_token: String
}

#[derive(Deserialize)]
pub struct UserForm {
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub access_token: String
}

impl User {
    pub fn all(conn: &SqliteConnection) -> Vec<User> {
        all_users.order(users::id.desc()).load::<User>(conn).unwrap()
    }

    pub fn insertOrUpdate(user_form: UserForm, conn: &SqliteConnection) -> bool {
        let t = User { id: None, email: user_form.email, first_name: user_form.first_name, last_name: user_form.last_name, access_token: user_form.access_token };
        diesel::replace_into(users::table).values(&t).execute(conn).is_ok()
    }

    pub fn delete_with_id(id: i32, conn: &SqliteConnection) -> bool {
        diesel::delete(all_users.find(id)).execute(conn).is_ok()
    }

    #[cfg(test)]
    pub fn delete_all(conn: &SqliteConnection) -> bool {
        diesel::delete(all_users).execute(conn).is_ok()
    }
}
