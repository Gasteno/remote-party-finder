use crate::listing_container::{ListingContainer, QueriedListing};
use chrono::{TimeDelta, Utc};
use futures_util::StreamExt;
use mongodb::bson::doc;
use mongodb::Collection;

pub async fn get_current_listings(
    collection: Collection<ListingContainer>,
) -> anyhow::Result<Vec<QueriedListing>> {
    let two_hours_ago = Utc::now() - TimeDelta::try_hours(2).unwrap();
    let cursor = collection
        .aggregate(
            [
                // don't ask me why, but mongo shits itself unless you provide a hard date
                // doc! {
                //     "$match": {
                //         "created_at": {
                //             "$gte": {
                //                 "$dateSubtract": {
                //                     "startDate": "$$NOW",
                //                     "unit": "hour",
                //                     "amount": 2,
                //                 },
                //             },
                //         },
                //     }
                // },
                doc! {
                    "$match": {
                        "updated_at": { "$gte": two_hours_ago },
                    }
                },
                doc! {
                    "$match": {
                        // filter private pfs
                        "listing.search_area": { "$bitsAllClear": 2 },
                    }
                },
                doc! {
                    "$set": {
                        "time_left": {
                            "$divide": [
                                {
                                    "$subtract": [
                                        { "$multiply": ["$listing.seconds_remaining", 1000] },
                                        { "$subtract": ["$$NOW", "$updated_at"] },
                                    ]
                                },
                                1000,
                            ]
                        },
                        "updated_minute": {
                            "$dateTrunc": {
                                "date": "$updated_at",
                                "unit": "minute",
                                "binSize": 5,
                            },
                        },
                    }
                },
                doc! {
                    "$match": {
                        "time_left": { "$gte": 0 },
                    }
                },
            ],
            None,
        )
        .await?;

    let collect = cursor
        .filter_map(async |res| res.ok()
            .and_then(|doc| mongodb::bson::from_document(doc).ok()))
        .collect::<Vec<_>>()
        .await;

    Ok(collect)
}
