use anyhow::Result;
use chrono::{TimeDelta, Utc};
use mongodb::bson::{Document, doc};
use mongodb::options::AggregateOptions;
use futures_util::TryStreamExt;
use crate::stats::{Aliases, Statistics};
use crate::web::State;

lazy_static::lazy_static! {
    static ref QUERY: [Document; 2] = [
        doc! {
            "$match": {
                // filter private pfs
                "listing.search_area": { "$bitsAllClear": 2 },
            }
        },
        doc! {
            "$facet": {
                "count": [
                    {
                        "$count": "count",
                    },
                ],
                "duties": [
                    {
                        "$group": {
                            "_id": [
                                "$listing.duty_type",
                                "$listing.category",
                                "$listing.duty",
                            ],
                            "count": {
                                "$sum": 1
                            },
                        }
                    },
                    {
                        "$sort": {
                            "count": -1,
                        }
                    }
                ],
                "hosts": [
                    {
                        "$group": {
                            "_id": {
                                "world": "$listing.created_world",
                                "content_id": "$listing.content_id_lower",
                            },
                            "count": { "$sum": 1 },
                        }
                    },
                    {
                        "$sort": {
                            "count": -1,
                        }
                    },
                    {
                        "$group": {
                            "_id": "$_id.world",
                            "count": {
                                "$sum": "$count",
                            },
                            "content_ids": {
                                "$push": {
                                    "content_id": "$_id.content_id",
                                    "count": "$count",
                                }
                            }
                        }
                    },
                    {
                        "$addFields": {
                            "content_ids": {
                                "$slice": ["$content_ids", 0, 15],
                            },
                        }
                    },
                    {
                        "$sort": { "count": -1 }
                    },
                ],
                "hours": [
                    {
                        "$group": {
                            "_id": {
                                "$hour": "$created_at",
                            },
                            "count": {
                                "$sum": 1
                            },
                        }
                    },
                    {
                        "$sort": {
                            "_id": 1,
                        }
                    }
                ],
                "days": [
                    {
                        "$group": {
                            "_id": {
                                "$dayOfWeek": "$created_at",
                            },
                            "count": {
                                "$sum": 1
                            },
                        }
                    },
                    {
                        "$sort": {
                            "_id": 1,
                        }
                    }
                ],
            }
        },
    ];

    static ref ALIASES_QUERY: [Document; 1] = [
        doc! {
            "$facet": {
                "aliases": [
                    {
                        "$sort": {
                            "created_at": -1,
                        }
                    },
                    {
                        "$group": {
                            "_id": "$listing.content_id_lower",
                            "alias": {
                                "$first": {
                                    "name": "$listing.name",
                                    "home_world": "$listing.home_world",
                                },
                            },
                        }
                    }
                ],
            },
        },
    ];
}

pub async fn get_stats(state: &State) -> Result<Statistics> {
    get_stats_internal(state, QUERY.iter().cloned()).await
}

pub async fn get_stats_seven_days(state: &State) -> Result<Statistics> {
    let last_week = Utc::now() - TimeDelta::try_days(7).unwrap();

    let mut docs = QUERY.to_vec();
    docs.insert(0, doc! {
        "$match": {
            "created_at": {
                "$gte": last_week,
            },
        },
    });

    get_stats_internal(state, docs).await
}

async fn get_stats_internal(state: &State, docs: impl IntoIterator<Item = Document>) -> Result<Statistics> {
    let mut cursor = state
        .collection()
        .aggregate(docs, AggregateOptions::builder()
            .allow_disk_use(true)
            .build())
        .await?;
    let doc = cursor.try_next().await?;
    let doc = doc.ok_or_else(|| anyhow::anyhow!("missing document"))?;
    let mut stats: Statistics = mongodb::bson::from_document(doc)?;

    let ids: Vec<u32> = stats.hosts.iter().flat_map(|host| host.content_ids.iter().map(|entry| entry.content_id)).collect();
    let mut aliases_query: Vec<Document> = ALIASES_QUERY.iter().cloned().collect();
    aliases_query.insert(0, doc! {
        "$match": {
            "listing.content_id_lower": {
                "$in": ids,
            }
        }
    });
    let mut cursor = state
        .collection()
        .aggregate(aliases_query, AggregateOptions::builder()
            .allow_disk_use(true)
            .build())
        .await?;
    let doc = cursor.try_next().await?;
    let doc = doc.ok_or_else(|| anyhow::anyhow!("missing document"))?;
    let aliases: Aliases = mongodb::bson::from_document(doc)?;

    stats.aliases = aliases.aliases;

    Ok(stats)
}
