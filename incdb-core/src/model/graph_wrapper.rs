//! Graph Wrapper
//!
//! EventSourcedGraphとWorldGraphを統一的に扱うためのラッパー

use crate::model::{EventSourcedGraph, IId, Incidence, Level, RoleId, WorldGraph};
use std::sync::{Arc, Mutex};
use tokio::runtime::Runtime;

/// グラフの統一インターフェース
pub enum GraphWrapper {
    WorldGraph(Arc<Mutex<WorldGraph>>),
    EventSourcedGraph(Arc<tokio::sync::Mutex<EventSourcedGraph>>),
}

impl GraphWrapper {
    /// WorldGraphから作成
    pub fn from_world_graph(graph: Arc<Mutex<WorldGraph>>) -> Self {
        Self::WorldGraph(graph)
    }

    /// EventSourcedGraphから作成
    pub fn from_event_sourced_graph(graph: Arc<tokio::sync::Mutex<EventSourcedGraph>>) -> Self {
        Self::EventSourcedGraph(graph)
    }

    /// Incidenceを取得（同期）
    pub fn get(&self, id: IId) -> Option<Incidence> {
        match self {
            Self::WorldGraph(graph) => {
                let g = graph.lock().unwrap();
                g.get(id).cloned()
            }
            Self::EventSourcedGraph(graph) => {
                // 非同期を同期に変換
                let rt = Runtime::new().ok()?;
                rt.block_on(async {
                    let g = graph.lock().await;
                    g.get(id).await.ok().flatten()
                })
            }
        }
    }

    /// Incidenceを追加（同期）
    pub fn add_incidence(&self, mut incidence: Incidence) -> IId {
        match self {
            Self::WorldGraph(graph) => {
                let mut g = graph.lock().unwrap();
                if incidence.id.0 == 0 {
                    incidence.id = g.new_id();
                }
                g.add_incidence(incidence);
                incidence.id
            }
            Self::EventSourcedGraph(graph) => {
                // 非同期を同期に変換
                let rt = Runtime::new().unwrap();
                rt.block_on(async {
                    let mut g = graph.lock().await;
                    if incidence.id.0 == 0 {
                        incidence.id = g.new_id();
                    }
                    g.add_incidence(incidence, None).await.unwrap_or(incidence.id)
                })
            }
        }
    }

    /// すべてのIncidenceをイテレート（同期）
    pub fn iter(&self) -> Vec<Incidence> {
        match self {
            Self::WorldGraph(graph) => {
                let g = graph.lock().unwrap();
                g.iter().cloned().collect()
            }
            Self::EventSourcedGraph(graph) => {
                // 非同期を同期に変換
                let rt = Runtime::new().unwrap();
                rt.block_on(async {
                    let g = graph.lock().await;
                    g.iter().await.unwrap_or_default()
                })
            }
        }
    }

    /// グラフのサイズ
    pub fn len(&self) -> usize {
        match self {
            Self::WorldGraph(graph) => {
                let g = graph.lock().unwrap();
                g.len()
            }
            Self::EventSourcedGraph(graph) => {
                // 非同期を同期に変換
                let rt = Runtime::new().unwrap();
                rt.block_on(async {
                    let g = graph.lock().await;
                    g.len().await.unwrap_or(0)
                })
            }
        }
    }

    /// 空かどうか
    pub fn is_empty(&self) -> bool {
        match self {
            Self::WorldGraph(graph) => {
                let g = graph.lock().unwrap();
                g.is_empty()
            }
            Self::EventSourcedGraph(graph) => {
                // 非同期を同期に変換
                let rt = Runtime::new().unwrap();
                rt.block_on(async {
                    let g = graph.lock().await;
                    g.is_empty().await.unwrap_or(true)
                })
            }
        }
    }

    /// 新しいIDを生成
    pub fn new_id(&self) -> IId {
        match self {
            Self::WorldGraph(graph) => {
                let mut g = graph.lock().unwrap();
                g.new_id()
            }
            Self::EventSourcedGraph(graph) => {
                // 非同期を同期に変換
                let rt = Runtime::new().unwrap();
                rt.block_on(async {
                    let mut g = graph.lock().await;
                    g.new_id()
                })
            }
        }
    }
}

