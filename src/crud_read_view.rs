use crud_shared_types::{Condition, ConditionClause, ConditionElement};
use yew::{html::ChildrenRenderer, prelude::*};

use crate::{
    crud_instance::Item,
    services::crud_rest_data_provider::{CrudRestDataProvider, ReadOne},
};

use super::{prelude::*, types::RequestError};

pub enum Msg<T: CrudMainTrait> {
    Back,
    LoadedEntity(Result<Option<T::ReadModel>, RequestError>),
}

#[derive(Properties, PartialEq)]
pub struct Props<T: CrudMainTrait> {
    pub children: ChildrenRenderer<Item>,
    pub data_provider: CrudRestDataProvider<T>,
    pub config: CrudInstanceConfig<T>,
    pub id: u32,
    pub list_view_available: bool,
    pub on_list_view: Callback<()>,
}

pub struct CrudReadView<T: CrudMainTrait> {
    entity: Result<T::UpdateModel, NoData>,
}

impl<T: 'static + CrudMainTrait> Component for CrudReadView<T> {
    type Message = Msg<T>;
    type Properties = Props<T>;

    fn create(ctx: &Context<Self>) -> Self {
        let id = ctx.props().id;
        let data_provider = ctx.props().data_provider.clone();
        ctx.link()
            .send_future(async move { Msg::LoadedEntity(load_entity(data_provider, id).await) });
        Self {
            entity: Err(NoData::NotYetLoaded),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Back => {
                ctx.props().on_list_view.emit(());
                false
            }
            Msg::LoadedEntity(data) => {
                self.entity = match data {
                    Ok(data) => match data {
                        Some(entity) => Ok(entity.into()),
                        None => Err(NoData::FetchReturnedNothing),
                    },
                    Err(err) => Err(NoData::FetchFailed(err)),
                };
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        // NOTE: We only not use <CrudFields<T::ReadModel>, as we would then also have to specify config.elements to contain ReadModel fields, which it currently does not do...
        // Idea for  future lukas: We could support both elements_read and elements_update, but this requires more work when setting up an instance (probably all duplicated..).
        html! {
            <div>
                {
                    match &self.entity {
                        Ok(entity) => {
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

                                <CrudFields<T::UpdateModel>
                                    api_base_url={ctx.props().config.api_base_url.clone()}
                                    children={ctx.props().children.clone()}
                                    elements={ctx.props().config.elements.clone()}
                                    entity={entity.clone()}
                                    mode={FieldMode::Readable}
                                    current_view={CrudView::Read(ctx.props().id)}
                                    value_changed={|_| {}}
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
                                    {format!("Keine Daten verfügbar: {:?}", reason)}
                                </div>
                                </>
                            }
                        }
                    }
                }
            </div>
        }
    }
}

pub async fn load_entity<T: CrudMainTrait>(
    data_provider: CrudRestDataProvider<T>,
    id: u32,
) -> Result<Option<T::ReadModel>, RequestError> {
    data_provider
        .read_one(ReadOne {
            skip: None,
            order_by: None,
            condition: Some(Condition::All(vec![ConditionElement::Clause(
                ConditionClause {
                    column_name: String::from(T::ReadModel::get_id_field().get_name()),
                    operator: crud_shared_types::Operator::Equal,
                    value: crud_shared_types::ConditionClauseValue::U32(id),
                },
            )])),
        })
        .await
}
