//! Leitura segura da tela de Anexação de Processos (#270).
//! Oráculo: Todos src/todos/anexacao.py (UnB 4.0.9, 15/10/2026).
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct TelaDeAnexacao {
    pub(crate) anexados: Vec<String>,
}
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Destino {
    pub(crate) id: String,
    pub(crate) nup: String,
    pub(crate) tipo: String,
}
pub(crate) fn ler_tela(_html: &str) -> Result<TelaDeAnexacao, &'static str> {
    Err("tela_de_anexacao_nao_implementada")
}
pub(crate) fn ler_destino(_xml: &str) -> Result<Destino, &'static str> {
    Err("destino_a_anexar_nao_implementado")
}
pub(crate) fn ler_confirmacao(_xml: &str) -> Result<bool, &'static str> {
    Err("verificacao_da_anexacao_nao_implementada")
}
pub(crate) fn decidir(
    _principal: &str, _id_principal: &str, _anexar: &Destino,
    _existentes: &[String],
) -> Result<bool, &'static str> {
    Err("decisao_de_anexacao_nao_implementada")
}
#[cfg(test)]
#[path = "processo_anexar/tests.rs"]
mod tests;
