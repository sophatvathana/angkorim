use std::collections::{ HashMap, HashSet };
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct GroupMember {
    pub user_id: String,
    pub role: MemberRole,
    pub joined_at: i64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MemberRole {
    Owner,
    Admin,
    Member,
}

#[derive(Debug, Clone)]
pub struct Group {
    pub id: String,
    pub name: String,
    pub description: String,
    pub created_at: i64,
    pub owner_id: String,
    pub members: HashMap<String, GroupMember>,
}

#[derive(Debug, Clone)]
pub struct Channel {
    pub id: String,
    pub name: String,
    pub description: String,
    pub created_at: i64,
    pub owner_id: String,
    pub subscribers: HashSet<String>,
}

pub struct GroupManager {
    groups: Arc<RwLock<HashMap<String, Group>>>,
    channels: Arc<RwLock<HashMap<String, Channel>>>,
}

impl GroupManager {
    pub fn new() -> Self {
        Self {
            groups: Arc::new(RwLock::new(HashMap::new())),
            channels: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    // Group management
    pub async fn create_group(&self, name: String, description: String, owner_id: String) -> Group {
        let group = Group {
            id: Uuid::new_v4().to_string(),
            name,
            description,
            created_at: chrono::Utc::now().timestamp(),
            owner_id: owner_id.clone(),
            members: {
                let mut members = HashMap::new();
                members.insert(owner_id.clone(), GroupMember {
                    user_id: owner_id,
                    role: MemberRole::Owner,
                    joined_at: chrono::Utc::now().timestamp(),
                });
                members
            },
        };

        self.groups.write().await.insert(group.id.clone(), group.clone());
        group
    }

    pub async fn add_member(&self, group_id: &str, user_id: &str, role: MemberRole) -> bool {
        if let Some(group) = self.groups.write().await.get_mut(group_id) {
            group.members.insert(user_id.to_string(), GroupMember {
                user_id: user_id.to_string(),
                role,
                joined_at: chrono::Utc::now().timestamp(),
            });
            true
        } else {
            false
        }
    }

    pub async fn remove_member(&self, group_id: &str, user_id: &str) -> bool {
        if let Some(group) = self.groups.write().await.get_mut(group_id) {
            group.members.remove(user_id);
            true
        } else {
            false
        }
    }

    pub async fn get_group(&self, group_id: &str) -> Option<Group> {
        self.groups.read().await.get(group_id).cloned()
    }

    pub async fn get_user_groups(&self, user_id: &str) -> Vec<Group> {
        self.groups
            .read()
            .await
            .values()
            .filter(|group| group.members.contains_key(user_id))
            .cloned()
            .collect()
    }

    pub async fn update_member_role(&self, group_id: &str, user_id: &str, new_role: MemberRole) -> bool {
        if let Some(group) = self.groups.write().await.get_mut(group_id) {
            if let Some(member) = group.members.get_mut(user_id) {
                member.role = new_role;
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    // Channel management
    pub async fn create_channel(&self, name: String, description: String, owner_id: String) -> Channel {
        let channel = Channel {
            id: Uuid::new_v4().to_string(),
            name,
            description,
            created_at: chrono::Utc::now().timestamp(),
            owner_id,
            subscribers: HashSet::new(),
        };

        self.channels.write().await.insert(channel.id.clone(), channel.clone());
        channel
    }

    pub async fn add_subscriber(&self, channel_id: &str, user_id: &str) -> bool {
        if let Some(channel) = self.channels.write().await.get_mut(channel_id) {
            channel.subscribers.insert(user_id.to_string());
            true
        } else {
            false
        }
    }

    pub async fn remove_subscriber(&self, channel_id: &str, user_id: &str) -> bool {
        if let Some(channel) = self.channels.write().await.get_mut(channel_id) {
            channel.subscribers.remove(user_id);
            true
        } else {
            false
        }
    }

    pub async fn get_channel(&self, channel_id: &str) -> Option<Channel> {
        self.channels.read().await.get(channel_id).cloned()
    }

    pub async fn get_user_channels(&self, user_id: &str) -> Vec<Channel> {
        self.channels
            .read()
            .await
            .values()
            .filter(|channel| channel.subscribers.contains(user_id))
            .cloned()
            .collect()
    }
}
