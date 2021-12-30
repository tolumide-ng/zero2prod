use crate::domain::{
    subscriber_name::SubscriberName,
    subscriber_email::SubscriberEmail,
};

pub struct NewSubscriber {
    pub email: SubscriberEmail,
    pub name: SubscriberName,
}
