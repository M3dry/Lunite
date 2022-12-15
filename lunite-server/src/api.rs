use crate::todos::{Profile, Profiles, Todo};

use actix_web::http::header::ContentType;
use actix_web::{delete, get, post, put, web, HttpResponse, Responder, ResponseError};

use serde::Serialize;

use std::path::PathBuf;
use std::{fmt::Display, sync::Mutex};

#[derive(Debug, Serialize)]
pub struct ErrIdx {
    idx: usize,
    err: String,
}

impl Display for ErrIdx {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl ResponseError for ErrIdx {}

#[get("/api")]
pub async fn get_api(data: web::Data<Mutex<Profiles>>) -> impl Responder {
    HttpResponse::Ok().content_type(ContentType::json()).body(
        serde_json::to_string(
            &(*match data.lock() {
                Ok(profiles) => profiles,
                Err(_) => return HttpResponse::Conflict().finish(),
            }),
        )
        .unwrap(),
    )
}

#[post("/api")]
pub async fn post_api(req: web::Json<Profile>, data: web::Data<Mutex<Profiles>>) -> impl Responder {
    match data.lock() {
        Ok(mut profiles) => profiles.add(req.0),
        Err(_) => return HttpResponse::Conflict().finish(),
    }

    HttpResponse::Ok().finish()
}

#[get("/api/current")]
pub async fn get_api_current(data: web::Data<Mutex<Profiles>>) -> impl Responder {
    HttpResponse::Ok().content_type(ContentType::json()).body(
        serde_json::to_string(&match data.lock() {
            Ok(profiles) => profiles.current(),
            Err(_) => return HttpResponse::Conflict().finish(),
        })
        .unwrap(),
    )
}

#[put("/api/current")]
pub async fn put_api_current(
    req: web::Json<usize>,
    data: web::Data<Mutex<Profiles>>,
) -> Result<impl Responder, ErrIdx> {
    match data.lock() {
        Ok(mut profiles) => {
            if req.0 > profiles.len() || req.0 < 1 {
                return Err(ErrIdx {
                    idx: req.0,
                    err: String::from("Err: Idx out of bounds"),
                });
            } else {
                profiles.change(req.0);
            }

            profiles.save();
        }
        Err(err) => {
            return Err(ErrIdx {
                idx: req.0,
                err: format!("Err: Mutex lock - {}", err),
            })
        }
    };

    Ok(HttpResponse::Ok().finish())
}

#[get("/api/{idx}")]
pub async fn get_api_idx(
    idx: web::Path<usize>,
    data: web::Data<Mutex<Profiles>>,
) -> Result<impl Responder, ErrIdx> {
    // match data.lock() {
    //     Ok(profiles) => Ok(HttpResponse::Ok().content_type(ContentType::json()).body(
    //         serde_json::to_string(if *id == 0 {
    //             profiles.get_current()
    //         } else {
    //             match profiles.nth(*id) {
    //                 Some(profile) => profile,
    //                 None => {
    //                     return Err(ErrIdx {
    //                         idx: *id,
    //                         err: String::from("Err: Idx out of bounds"),
    //                     })
    //                 }
    //             }
    //         })
    //         .unwrap(),
    //     )),
    //     Err(err) => {
    //         return Err(ErrIdx {
    //             idx: *id,
    //             err: format!("Mutex lock error {}", err),
    //         })
    //     }
    // }

    get_profile(*idx, data, |profile| {
        Ok(HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(serde_json::to_string(profile).unwrap()))
    })
}

#[put("/api/{idx}")]
pub async fn put_api_idx(
    idx: web::Path<usize>,
    req: web::Json<Profile>,
    data: web::Data<Mutex<Profiles>>,
) -> Result<impl Responder, ErrIdx> {
    // match data.lock() {
    //     Ok(mut profiles) => {
    //         if *id == 0 {
    //             *profiles.get_current_mut() = req.0;
    //         } else {
    //             match profiles.nth_mut(*id) {
    //                 Some(profile) => *profile = req.0,
    //                 None => {
    //                     return Err(ErrIdx {
    //                         idx: *id,
    //                         err: String::from("Err: Idx out of bounds"),
    //                     })
    //                 }
    //             }
    //         }

    //         profiles.save();
    //     }
    //     Err(err) => {
    //         return Err(ErrIdx {
    //             idx: *id,
    //             err: format!("Err: Mutex lock - {}", err),
    //         })
    //     }
    // }

    // Ok(HttpResponse::Ok().finish())

    get_profile_mut_req(*idx, data, req, |profile, req| {
        *profile = req.0;
        Ok(HttpResponse::Ok().finish())
    })
}

#[delete("/api/{idx}")]
pub async fn delete_api_idx(
    idx: web::Path<usize>,
    data: web::Data<Mutex<Profiles>>,
) -> Result<impl Responder, ErrIdx> {
    match data.lock() {
        Ok(mut profiles) => {
            match profiles.len() {
                len if len == 1 => {
                    return Err(ErrIdx {
                        idx: *idx,
                        err: String::from("Err: Last Profile"),
                    })
                }
                len if len < *idx => {
                    return Err(ErrIdx {
                        idx: *idx,
                        err: String::from("Err: Idx out of bounds"),
                    })
                }
                _ => {
                    if *idx == 0 {
                        let cur = profiles.current() - 1;
                        profiles.get_all_mut().remove(cur);
                        if cur != 0 {
                            profiles.change(cur);
                        }
                    } else {
                        profiles.get_all_mut().remove(*idx - 1);
                        let current = profiles.current();
                        if current > *idx {
                            profiles.change(current - 1);
                        }
                    }
                }
            }

            profiles.save()
        }
        Err(err) => {
            return Err(ErrIdx {
                idx: *idx,
                err: format!("Err: Mutex lock - {}", err),
            })
        }
    }

    Ok(HttpResponse::Ok().finish())
}

#[get("/api/{idx}/name")]
pub async fn get_api_idx_name(
    idx: web::Path<usize>,
    data: web::Data<Mutex<Profiles>>,
) -> Result<impl Responder, ErrIdx> {
    // match data.lock() {
    //     Ok(profiles) => Ok(HttpResponse::Ok().content_type(ContentType::json()).body(
    //         serde_json::to_string(if *idx == 0 {
    //             &profiles.get_current().name
    //         } else {
    //             match profiles.nth(*idx) {
    //                 Some(profile) => &profile.name,
    //                 None => {
    //                     return Err(ErrIdx {
    //                         idx: *idx,
    //                         err: String::from("Err: Idx out of bounds"),
    //                     })
    //                 }
    //             }
    //         })
    //         .unwrap(),
    //     )),
    //     Err(err) => {
    //         return Err(ErrIdx {
    //             idx: *idx,
    //             err: format!("Mutex lock error {}", err),
    //         })
    //     }
    // }

    get_profile(*idx, data, |profile| {
        Ok(HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(serde_json::to_string(&profile.name).unwrap()))
    })
}

#[put("/api/{idx}/name")]
pub async fn put_api_idx_name(
    idx: web::Path<usize>,
    req: web::Json<String>,
    data: web::Data<Mutex<Profiles>>,
) -> Result<impl Responder, ErrIdx> {
    // match data.lock() {
    //     Ok(mut profiles) => {
    //         if *idx == 0 {
    //             profiles.get_current_mut().name = req.0;
    //         } else {
    //             match profiles.nth_mut(*idx) {
    //                 Some(profile) => profile.name = req.0,
    //                 None => {
    //                     return Err(ErrIdx {
    //                         idx: *idx,
    //                         err: String::from("Err: Idx out of bounds"),
    //                     })
    //                 }
    //             }
    //         }
    //         profiles.save();

    //         Ok(HttpResponse::Ok().finish())
    //     }
    //     Err(err) => {
    //         return Err(ErrIdx {
    //             idx: *idx,
    //             err: format!("Mutex lock error {}", err),
    //         })
    //     }
    // }

    get_profile_mut_req(*idx, data, req, |profile, req| {
        profile.name = req.0;
        Ok(HttpResponse::Ok().finish())
    })
}

#[get("/api/{idx}/todos")]
pub async fn get_api_idx_todos(
    idx: web::Path<usize>,
    data: web::Data<Mutex<Profiles>>,
) -> Result<impl Responder, ErrIdx> {
    // match data.lock() {
    //     Ok(profiles) => {
    //         let todos = if *idx == 0 {
    //             profiles.get_current().iter()
    //         } else {
    //             match profiles.nth(*idx) {
    //                 Some(profile) => profile.iter(),
    //                 None => {
    //                     return Err(ErrIdx {
    //                         idx: *idx,
    //                         err: String::from("Err: Idx out of bounds"),
    //                     })
    //                 }
    //             }
    //         }
    //         .collect::<Vec<(&Vec<crate::todos::Todo>, Option<(&String, &PathBuf)>)>>();

    //         Ok(HttpResponse::Ok()
    //             .content_type(ContentType::json())
    //             .body(serde_json::to_string(&todos).unwrap()))
    //     }
    //     Err(err) => {
    //         return Err(ErrIdx {
    //             idx: *idx,
    //             err: format!("Mutex lock error {}", err),
    //         })
    //     }
    // }

    get_profile(*idx, data, |profile| {
        Ok(HttpResponse::Ok().content_type(ContentType::json()).body(
            serde_json::to_string(
                &profile
                    .iter()
                    .map(|t| t.0)
                    .flatten()
                    .collect::<Vec<&Todo>>(),
            )
            .unwrap(),
        ))
    })
}

#[post("/api/{idx}/todos")]
pub async fn post_api_idx_todos(
    idx: web::Path<usize>,
    req: web::Json<Todo>,
    data: web::Data<Mutex<Profiles>>,
) -> Result<impl Responder, ErrIdx> {
    get_profile_mut_req(*idx, data, req, |profile, req| {
        profile.iter_mut().next().unwrap().0.push(req.0);
        Ok(HttpResponse::Ok().finish())
    })
}

#[delete("/api/{idx}/todos/done")]
pub async fn delete_api_idx_todos_done(
    idx: web::Path<usize>,
    data: web::Data<Mutex<Profiles>>,
) -> Result<impl Responder, ErrIdx> {
    get_profile_mut(*idx, data, move |profile| {
        profile.remove_done_todo();
        Ok(HttpResponse::Ok().finish())
    })
}

#[get("/api/{idx}/todos/{idx1}")]
pub async fn get_api_idx_todos_idx1(
    idx: web::Path<(usize, usize)>,
    data: web::Data<Mutex<Profiles>>,
) -> Result<impl Responder, ErrIdx> {
    get_profile(idx.0, data, move |profile| match &profile.nth_todo(idx.1) {
        Some(todo) => Ok(HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(serde_json::to_string(&todo).unwrap())),
        None => Err(ErrIdx {
            idx: idx.1,
            err: String::from("Err: Second index out of bounds"),
        }),
    })
}

#[put("/api/{idx}/todos/{idx1}")]
pub async fn put_api_idx_todos_idx1(
    idx: web::Path<(usize, usize)>,
    req: web::Json<Todo>,
    data: web::Data<Mutex<Profiles>>,
) -> Result<impl Responder, ErrIdx> {
    get_profile_mut_req(idx.0, data, req, move |profile, req| {
        match profile.nth_todo_mut(idx.1) {
            Some(todo) => {
                *todo = req.0;
                Ok(HttpResponse::Ok().finish())
            }
            None => Err(ErrIdx {
                idx: idx.1,
                err: String::from("Err: Second index out of bounds"),
            }),
        }
    })
}

#[delete("/api/{idx}/todos/{idx1}")]
pub async fn delete_api_idx_todos_idx1(
    idx: web::Path<(usize, usize)>,
    data: web::Data<Mutex<Profiles>>,
) -> Result<impl Responder, ErrIdx> {
    get_profile_mut(idx.0, data, move |profile| {
        match profile.remove_todo(idx.1) {
            Ok(_) => Ok(HttpResponse::Ok().finish()),
            Err(_) => Err(ErrIdx {
                idx: idx.1,
                err: String::from("Err: Second index out of bounds"),
            }),
        }
    })
}

#[derive(Serialize)]
pub struct Link<'a> {
    name: &'a String,
    path: &'a PathBuf,
    local_todos: &'a Vec<crate::todos::Todo>,
}

#[get("/api/{idx}/links")]
pub async fn get_api_idx_links(
    idx: web::Path<usize>,
    data: web::Data<Mutex<Profiles>>,
) -> Result<impl Responder, ErrIdx> {
    // match data.lock() {
    //     Ok(profiles) => {
    //         let todos = if *idx == 0 {
    //             profiles.get_current().iter()
    //         } else {
    //             match profiles.nth(*idx) {
    //                 Some(profile) => profile.iter(),
    //                 None => {
    //                     return Err(ErrIdx {
    //                         idx: *idx,
    //                         err: String::from("Err: Idx out of bounds"),
    //                     })
    //                 }
    //             }
    //         }
    //         .filter(|v| v.1 != None)
    //         .map(|v| {
    //             let unwrapped = v.1.unwrap();
    //             Link {
    //                 name: unwrapped.0,
    //                 path: unwrapped.1,
    //                 local_todos: v.0,
    //             }
    //         })
    //         .collect::<Vec<Link>>();

    //         Ok(HttpResponse::Ok()
    //             .content_type(ContentType::json())
    //             .body(serde_json::to_string(&todos).unwrap()))
    //     }
    //     Err(err) => {
    //         return Err(ErrIdx {
    //             idx: *idx,
    //             err: format!("Mutex lock error {}", err),
    //         })
    //     }
    // }

    get_profile(*idx, data, |profile| {
        Ok(HttpResponse::Ok().content_type(ContentType::json()).body(
            serde_json::to_string(
                &profile
                    .iter()
                    .filter(|v| v.1 != None)
                    .map(|v| {
                        let unwrapped = v.1.unwrap();
                        Link {
                            name: unwrapped.0,
                            path: unwrapped.1,
                            local_todos: v.0,
                        }
                    })
                    .collect::<Vec<Link>>(),
            )
            .unwrap(),
        ))
    })
}

#[post("/api/{idx}/links")]
pub async fn post_api_idx_links(
    idx: web::Path<usize>,
    req: web::Json<crate::todos::Link>,
    data: web::Data<Mutex<Profiles>>,
) -> Result<impl Responder, ErrIdx> {
    // match data.lock() {
    //     Ok(mut profiles) => {
    //         if *idx == 0 {
    //             profiles.get_current_mut().add_link(req.0)
    //         } else {
    //             match profiles.nth_mut(*idx) {
    //                 Some(profile) => profile.add_link(req.0),
    //                 None => {
    //                     return Err(ErrIdx {
    //                         idx: *idx,
    //                         err: String::from("Err: Idx out of bounds"),
    //                     })
    //                 }
    //             }
    //         }

    //         Ok(HttpResponse::Ok().finish())
    //     }
    //     Err(err) => {
    //         return Err(ErrIdx {
    //             idx: *idx,
    //             err: format!("Mutex lock error {}", err),
    //         })
    //     }
    // }

    get_profile_mut_req(*idx, data, req, |profile, link| {
        profile.add_link(link.0);
        Ok(HttpResponse::Ok().finish())
    })
}

#[get("/api/{idx}/links/{idx1}")]
pub async fn get_api_idx_links_idx1(
    idx: web::Path<(usize, usize)>,
    data: web::Data<Mutex<Profiles>>,
) -> Result<impl Responder, ErrIdx> {
    get_profile(idx.0, data, move |profile| {
        if idx.1 == 0 {
            Err(ErrIdx {
                idx: idx.1,
                err: String::from("Err: Second index out of bounds"),
            })
        } else {
            match profile.nth_link(idx.1) {
                Some(link) => Ok(HttpResponse::Ok()
                    .content_type(ContentType::json())
                    .body(serde_json::to_string(link).unwrap())),
                None => Err(ErrIdx {
                    idx: idx.1,
                    err: String::from("Err: Second index out of bounds"),
                }),
            }
        }
    })
}

#[post("/api/{idx}/links/{idx1}")]
pub async fn post_api_idx_links_idx1(
    idx: web::Path<(usize, usize)>,
    req: web::Json<Todo>,
    data: web::Data<Mutex<Profiles>>,
) -> Result<impl Responder, ErrIdx> {
    get_profile_mut_req(idx.0, data, req, move |profile, req| {
        if idx.1 == 0 {
            Err(ErrIdx {
                idx: idx.1,
                err: String::from("Err: Second index out of bounds"),
            })
        } else {
            match profile.nth_link_mut(idx.1) {
                Some(link) => {
                    link.add_todo(req.0);
                    Ok(HttpResponse::Ok().finish())
                }
                None => Err(ErrIdx {
                    idx: idx.1,
                    err: String::from("Err: Second index out of bounds"),
                }),
            }
        }
    })
}

#[delete("/api/{idx}/links/{idx1}")]
pub async fn delete_api_idx_links_idx1(
    idx: web::Path<(usize, usize)>,
    data: web::Data<Mutex<Profiles>>,
) -> Result<impl Responder, ErrIdx> {
    get_profile_mut(idx.0, data, move |profile| {
        if idx.1 == 0 || profile.len_link() < idx.1 {
            Err(ErrIdx {
                idx: idx.1,
                err: String::from("Err: Second index out of bounds"),
            })
        } else {
            profile.remove_link(vec![idx.1]);
            Ok(HttpResponse::Ok().finish())
        }
    })
}

#[get("/api/{idx}/links/{idx1}/name")]
pub async fn get_api_idx_links_idx1_name(
    idx: web::Path<(usize, usize)>,
    data: web::Data<Mutex<Profiles>>,
) -> Result<impl Responder, ErrIdx> {
    get_profile(idx.0, data, move |profile| {
        if idx.1 == 0 {
            Err(ErrIdx {
                idx: idx.1,
                err: String::from("Err: Second index out of bounds"),
            })
        } else {
            match profile.nth_link(idx.1) {
                Some(link) => Ok(HttpResponse::Ok()
                    .content_type(ContentType::json())
                    .body(serde_json::to_string(&link.name).unwrap())),
                None => Err(ErrIdx {
                    idx: idx.1,
                    err: String::from("Err: Second index out of bounds"),
                }),
            }
        }
    })
}

#[put("/api/{idx}/links/{idx1}/name")]
pub async fn put_api_idx_links_idx1_name(
    idx: web::Path<(usize, usize)>,
    req: web::Json<String>,
    data: web::Data<Mutex<Profiles>>,
) -> Result<impl Responder, ErrIdx> {
    get_profile_mut_req(idx.0, data, req, move |profile, req| {
        if idx.1 == 0 {
            Err(ErrIdx {
                idx: idx.1,
                err: String::from("Err: Second index out of bounds"),
            })
        } else {
            match profile.nth_link_mut(idx.1) {
                Some(link) => {
                    link.name = req.0;
                    Ok(HttpResponse::Ok().finish())
                }
                None => Err(ErrIdx {
                    idx: idx.1,
                    err: String::from("Err: Second index out of bounds"),
                }),
            }
        }
    })
}

#[get("/api/{idx}/links/{idx1}/path")]
pub async fn get_api_idx_links_idx1_path(
    idx: web::Path<(usize, usize)>,
    data: web::Data<Mutex<Profiles>>,
) -> Result<impl Responder, ErrIdx> {
    get_profile(idx.0, data, move |profile| {
        if idx.1 == 0 {
            Err(ErrIdx {
                idx: idx.1,
                err: String::from("Err: Second index out of bounds"),
            })
        } else {
            match profile.nth_link(idx.1) {
                Some(link) => Ok(HttpResponse::Ok()
                    .content_type(ContentType::json())
                    .body(serde_json::to_string(&link.path).unwrap())),
                None => Err(ErrIdx {
                    idx: idx.1,
                    err: String::from("Err: Second index out of bounds"),
                }),
            }
        }
    })
}

#[put("/api/{idx}/links/{idx1}/path")]
pub async fn put_api_idx_links_idx1_path(
    idx: web::Path<(usize, usize)>,
    req: web::Json<PathBuf>,
    data: web::Data<Mutex<Profiles>>,
) -> Result<impl Responder, ErrIdx> {
    get_profile_mut_req(idx.0, data, req, move |profile, req| {
        if idx.1 == 0 {
            Err(ErrIdx {
                idx: idx.1,
                err: String::from("Err: Second index out of bounds"),
            })
        } else {
            match profile.nth_link_mut(idx.1) {
                Some(link) => {
                    link.path = req.0;
                    Ok(HttpResponse::Ok().finish())
                }
                None => Err(ErrIdx {
                    idx: idx.1,
                    err: String::from("Err: Second index out of bounds"),
                }),
            }
        }
    })
}

fn get_profile<F, R>(
    idx: usize,
    data: web::Data<Mutex<Profiles>>,
    func: F,
) -> Result<impl Responder, ErrIdx>
where
    F: Fn(&Profile) -> Result<R, ErrIdx>,
    R: Responder,
{
    match data.lock() {
        Ok(profiles) => {
            if idx == 0 {
                func(profiles.get_current())
            } else {
                match profiles.nth(idx) {
                    Some(profile) => func(profile),
                    None => Err(ErrIdx {
                        idx,
                        err: String::from("Err: Idx out of bounds"),
                    }),
                }
            }
        }
        Err(err) => Err(ErrIdx {
            idx,
            err: format!("Mutex lock error {}", err),
        }),
    }
}

fn get_profile_req<J, F, R>(
    idx: usize,
    data: web::Data<Mutex<Profiles>>,
    req: web::Json<J>,
    func: F,
) -> Result<impl Responder, ErrIdx>
where
    F: Fn(&Profile, web::Json<J>) -> Result<R, ErrIdx>,
    R: Responder,
{
    match data.lock() {
        Ok(profiles) => {
            if idx == 0 {
                func(profiles.get_current(), req)
            } else {
                match profiles.nth(idx) {
                    Some(profile) => func(profile, req),
                    None => Err(ErrIdx {
                        idx,
                        err: String::from("Err: Idx out of bounds"),
                    }),
                }
            }
        }
        Err(err) => Err(ErrIdx {
            idx,
            err: format!("Mutex lock error {}", err),
        }),
    }
}

fn get_profile_mut<F, R>(
    idx: usize,
    data: web::Data<Mutex<Profiles>>,
    mut func: F,
) -> Result<impl Responder, ErrIdx>
where
    F: FnMut(&mut Profile) -> Result<R, ErrIdx>,
    R: Responder,
{
    match data.lock() {
        Ok(mut profiles) => {
            let ret = if idx == 0 {
                func(profiles.get_current_mut())
            } else {
                match profiles.nth_mut(idx) {
                    Some(profile) => func(profile),
                    None => Err(ErrIdx {
                        idx,
                        err: String::from("Err: Idx out of bounds"),
                    }),
                }
            };
            profiles.save();

            ret
        }
        Err(err) => Err(ErrIdx {
            idx,
            err: format!("Mutex lock error {}", err),
        }),
    }
}

fn get_profile_mut_req<J, F, R>(
    idx: usize,
    data: web::Data<Mutex<Profiles>>,
    req: web::Json<J>,
    mut func: F,
) -> Result<impl Responder, ErrIdx>
where
    F: FnMut(&mut Profile, web::Json<J>) -> Result<R, ErrIdx>,
    R: Responder,
{
    match data.lock() {
        Ok(mut profiles) => {
            let ret = if idx == 0 {
                func(profiles.get_current_mut(), req)
            } else {
                match profiles.nth_mut(idx) {
                    Some(profile) => func(profile, req),
                    None => {
                        return Err(ErrIdx {
                            idx,
                            err: String::from("Err: Idx out of bounds"),
                        })
                    }
                }
            };

            profiles.save();
            return ret;
        }
        Err(err) => {
            return Err(ErrIdx {
                idx,
                err: format!("Mutex lock error {}", err),
            })
        }
    }
}
