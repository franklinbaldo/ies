//! Conferência do formulário de sobrestamento observado na UnB (#269).
//! Fonte: Todos tests/test_ciclo_sobrestar_010.py @323eb01.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ErroTela {
    NaoInterpretada,
    FormularioAusente,
    OutroProcesso,
    OpcaoNaoOferecida,
    DestinoPreenchido,
    CampoInesperado,
}
impl ErroTela {
    pub(crate) fn subcodigo(self) -> &'static str {
        match self {
            Self::NaoInterpretada => "tela_de_sobrestar_nao_interpretada",
            Self::FormularioAusente => "formulario_de_sobrestar_ausente",
            Self::OutroProcesso => "formulario_de_sobrestar_de_outro_processo",
            Self::OpcaoNaoOferecida => "opcao_somente_sobrestar_nao_oferecida",
            Self::DestinoPreenchido => "tela_de_sobrestar_com_destino_preenchido",
            Self::CampoInesperado => "campo_inesperado_na_tela_de_sobrestar",
        }
    }
}
pub(crate) fn conferir_formulario(_html: &str, _id: &str) -> Result<(), ErroTela> {
    Err(ErroTela::NaoInterpretada)
}
