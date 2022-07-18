use crud_shared_types::{
    Condition, ConditionClause, ConditionClauseValue, ConditionElement, Operator, SaveResult,
};
use yew::{
    html::{ChildrenRenderer, Scope},
    prelude::*,
};

use crate::{
    crud_instance::Item,
    services::crud_rest_data_provider::{CrudRestDataProvider, ReadOne, UpdateOne},
};

use super::{prelude::*, types::RequestError};

// TODO: CrudEditView tracks changes, but CrudCreateView does not. Consolidate this logic into a shared component.

pub enum Msg<T: CrudDataTrait> {
    Back,
    BackCanceled,
    BackApproved,
    LoadedEntity(Result<Option<T>, RequestError>),
    UpdatedEntity((Result<Option<SaveResult<T>>, RequestError>, Then)),
    Save,
    SaveAndReturn,
    SaveAndNew,
    Delete,
    ValueChanged((T::FieldType, Value)),
    GetInput((T::FieldType, Box<dyn FnOnce(Value)>)),
}

#[derive(Properties, PartialEq)]
pub struct Props<T: 'static + CrudDataTrait> {
    pub on_link: Callback<Option<Scope<CrudEditView<T>>>>,
    pub children: ChildrenRenderer<Item>,
    pub data_provider: CrudRestDataProvider<T>,
    pub config: CrudInstanceConfig<T>,
    pub id: u32,
    pub list_view_available: bool,
    pub on_saved: Callback<SaveResult<T>>,
    pub on_list: Callback<()>,
    pub on_create: Callback<()>,
    pub on_delete: Callback<T>,
}

pub struct CrudEditView<T: CrudDataTrait> {
    input: T,
    input_dirty: bool,
    user_wants_to_leave: bool,
    entity: Result<T, NoData>,
    ongoing_save: bool,
}

enum SetFrom {
    Fetch,
    Update,
}

pub enum Then {
    DoNothing,
    OpenListView,
    OpenCreateView,
}

impl<T: 'static + CrudDataTrait> CrudEditView<T> {
    // TODO: Remove this code duplication!
    
    fn set_entity(&mut self, data: Result<Option<T>, RequestError>, from: SetFrom) {
        self.entity = match data {
            Ok(data) => match data {
                Some(entity) => Ok(entity),
                None => Err(match from {
                    SetFrom::Fetch => NoData::FetchReturnedNothing,
                    SetFrom::Update => NoData::UpdateReturnedNothing,
                }),
            },
            Err(err) => Err(match from {
                SetFrom::Fetch => NoData::FetchFailed(err),
                SetFrom::Update => NoData::UpdateFailed(err),
            }),
        };
        if let Ok(entity) = &self.entity {
            self.input = entity.clone();
            self.input_dirty = false;
        }
    }

    fn set_entity_from_save_result(&mut self, data: Result<Option<SaveResult<T>>, RequestError>, from: SetFrom) {
        self.entity = match data {
            Ok(data) => match data {
                Some(save_result) => Ok(save_result.entity),
                None => Err(match from {
                    SetFrom::Fetch => NoData::FetchReturnedNothing,
                    SetFrom::Update => NoData::UpdateReturnedNothing,
                }),
            },
            Err(err) => Err(match from {
                SetFrom::Fetch => NoData::FetchFailed(err),
                SetFrom::Update => NoData::UpdateFailed(err),
            }),
        };
        if let Ok(entity) = &self.entity {
            self.input = entity.clone();
            self.input_dirty = false;
        }
    }

    fn save_entity(&self, ctx: &Context<Self>, and_then: Then) {
        let entity = self.input.clone();
        let id = ctx.props().id;
        let data_provider = ctx.props().data_provider.clone();
        // TODO: Like in create_view, store ongoing_save!!
        ctx.link().send_future(async move {
            Msg::UpdatedEntity((
                data_provider
                    .update_one(UpdateOne {
                        entity,
                        condition: Some(Condition::All(vec![ConditionElement::Clause(
                            ConditionClause {
                                column_name: T::get_id_field_name(),
                                operator: Operator::Equal,
                                value: ConditionClauseValue::U32(id),
                            },
                        )])),
                    })
                    .await,
                and_then,
            ))
        });
    }
}

impl<T: 'static + CrudDataTrait> Component for CrudEditView<T> {
    type Message = Msg<T>;
    type Properties = Props<T>;

    fn create(ctx: &Context<Self>) -> Self {
        ctx.props().on_link.emit(Some(ctx.link().clone()));
        let id = ctx.props().id;
        let data_provider = ctx.props().data_provider.clone();
        ctx.link()
            .send_future(async move { Msg::LoadedEntity(load_entity(data_provider, id).await) });
        Self {
            input: Default::default(),
            input_dirty: false,
            user_wants_to_leave: false,
            entity: Err(NoData::NotYetLoaded),
            ongoing_save: false,
        }
    }

    fn destroy(&mut self, ctx: &Context<Self>) {
        ctx.props().on_link.emit(None);
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Back => {
                self.user_wants_to_leave = true;
                match self.input_dirty {
                    true => {
                        // Waiting for the modal to be resolved!
                        true
                    }
                    false => {
                        ctx.props().on_list.emit(());
                        false
                    }
                }
            }
            Msg::BackCanceled => {
                self.user_wants_to_leave = false;
                true
            }
            Msg::BackApproved => {
                ctx.props().on_list.emit(());
                false
            }
            Msg::LoadedEntity(data) => {
                self.set_entity(data, SetFrom::Fetch);
                true
            }
            Msg::UpdatedEntity((data, and_then)) => {
                self.set_entity_from_save_result(data.clone(), SetFrom::Update);
                match data {
                    Ok(data) => match data {
                        Some(data) => {
                            ctx.props().on_saved.emit(data);
                            match and_then {
                                Then::DoNothing => {}
                                Then::OpenListView => ctx.props().on_list.emit(()),
                                Then::OpenCreateView => ctx.props().on_create.emit(()),
                            }
                        }
                        None => log::warn!("Could not update entity. Request returned nothing."),
                    },
                    Err(err) => log::warn!(
                        "Could not update entity due to RequestError: {}",
                        err.to_string()
                    ),
                }
                true
            }
            Msg::Save => {
                self.save_entity(ctx, Then::DoNothing);
                true
            }
            Msg::SaveAndReturn => {
                self.save_entity(ctx, Then::OpenListView);
                false
            }
            Msg::SaveAndNew => {
                self.save_entity(ctx, Then::OpenCreateView);
                false
            }
            Msg::Delete => {
                match &self.entity {
                    Ok(entity) => ctx.props().on_delete.emit(entity.clone()),
                    Err(_) => log::warn!(
                        "Cannot issue a delete event, as not entity is currently loaded!"
                    ),
                }
                false
            }
            Msg::ValueChanged((field, value)) => {
                field.set_value(&mut self.input, value);
                // We might only want to set this to true if the new value was actually different to the old value!
                match &self.entity {
                    Ok(entity) => self.input_dirty = &self.input != entity,
                    Err(_) => self.input_dirty = false,
                }
                false
            }
            Msg::GetInput((field, receiver)) => {
                receiver(field.get_value(&self.input));
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div>
                {
                    match &self.entity {
                        Ok(_entity) => {
                            html! {
                                <>
                                <div class={"crud-row crud-nav"}>
                                    <div class={"crud-col"}>
                                        <CrudBtnWrapper>
                                            <CrudBtn name={"Speichern"} variant={Variant::Primary} disabled={self.ongoing_save} onclick={ctx.link().callback(|_| Msg::Save)}>
                                                <CrudBtn name={"Speichern und zurück"} variant={Variant::Primary} disabled={self.ongoing_save} onclick={ctx.link().callback(|_| Msg::SaveAndReturn)} />
                                                <CrudBtn name={"Speichern und neu"} variant={Variant::Primary} disabled={self.ongoing_save} onclick={ctx.link().callback(|_| Msg::SaveAndNew)} />
                                            </CrudBtn>
                                            <CrudBtn name={"Löschen"} variant={Variant::Danger} disabled={self.ongoing_save} onclick={ctx.link().callback(|_| Msg::Delete)} />
                                        </CrudBtnWrapper>
                                    </div>

                                    <div class={"crud-col crud-col-flex-end"}>
                                        <CrudBtnWrapper>
                                            <CrudBtn name={"_back"} variant={Variant::Default} onclick={ctx.link().callback(|_| Msg::Back)}>
                                                <CrudBtnName>
                                                    <span style="text-decoration: underline;">{"L"}</span>{"istenansicht"}
                                                </CrudBtnName>
                                            </CrudBtn>
                                        </CrudBtnWrapper>
                                    </div>
                                </div>

                                <CrudFields<T>
                                    api_base_url={ctx.props().config.api_base_url.clone()}
                                    children={ctx.props().children.clone()}
                                    elements={ctx.props().config.elements.clone()}
                                    entity={self.input.clone()}
                                    mode={FieldMode::Editable}
                                    current_view={CrudView::Edit(ctx.props().id)}
                                    value_changed={ctx.link().callback(Msg::ValueChanged)}
                                />
                                </>
                            }
                        }
                        Err(reason) => {
                            html! {
                                <>
                                <div class={"crud-row crud-nav"}>
                                    <div class={"crud-col crud-col-flex-end"}>
                                        <CrudBtnWrapper>
                                            <CrudBtn name={"_back"} variant={Variant::Default} onclick={ctx.link().callback(|_| Msg::Back)}>
                                                <CrudBtnName>
                                                    <span style="text-decoration: underline;">{"L"}</span>{"istenansicht"}
                                                </CrudBtnName>
                                            </CrudBtn>
                                        </CrudBtnWrapper>
                                    </div>
                                </div>
                                <div>
                                    {format!("Daten nicht verfügbar: {:?}", reason)}
                                </div>
                                </>
                            }
                        }
                    }
                }
                if self.user_wants_to_leave {
                    <CrudModal>
                        <CrudLeaveModal
                            on_cancel={ctx.link().callback(|_| Msg::BackCanceled)}
                            on_leave={ctx.link().callback(|_| Msg::BackApproved)}
                        />
                    </CrudModal>
                }
            </div>
        }
    }
}

pub async fn load_entity<T: CrudDataTrait>(
    data_provider: CrudRestDataProvider<T>,
    id: u32,
) -> Result<Option<T>, RequestError> {
    data_provider
        .read_one(ReadOne {
            skip: None,
            order_by: None,
            condition: Some(Condition::All(vec![ConditionElement::Clause(
                ConditionClause {
                    column_name: T::get_id_field_name(),
                    operator: crud_shared_types::Operator::Equal,
                    value: crud_shared_types::ConditionClauseValue::U32(id),
                },
            )])),
        })
        .await
}
