//! Ações da barra do processo, derivadas da unidade e anotação no SQLite.
//! Links assinados aqui são estritamente sintéticos; não há credencial.
use iesei::ciencia_processo::Estado as Ciencia;
use iesei::processos::ProcessoSimulado;
use std::fmt::Write as _;

/// Renderiza apenas ações oferecidas pelo estado conhecido da origem.
pub(crate) fn acoes_do_processo(
    processo: &ProcessoSimulado,
    unidade_atual: &str,
    relacionamentos_observados: bool,
    atribuicao_observada: bool,
    prazo_observado: bool,
    ciencia: &Ciencia,
) -> String {
    let mut links = String::new();
    let id = &processo.id;
    let assinatura = crate::protocolo_sintetico::INFRA_HASH;
    if processo
        .aberto_em
        .iter()
        .any(|unidade| unidade == unidade_atual)
    {
        write!(
            links,
            r#"<a href="controlador.php?acao=procedimento_enviar&amp;acao_origem=arvore_visualizar&amp;id_procedimento={id}&amp;arvore=1&amp;infra_hash={assinatura}">Enviar</a>"#
        )
        .expect("formatação de String");
    }
    if processo.pode_incluir_documento(unidade_atual) {
        write!(
            links,
            r#"<a href="controlador.php?acao=documento_escolher_tipo&amp;id_procedimento={id}&amp;infra_hash={assinatura}">Incluir Documento</a>"#
        )
        .expect("formatação de String");
    }
    if matches!(ciencia, Ciencia::Pendente)
        && processo.aberto_em.iter().any(|u| u == unidade_atual)
    {
        links.push_str(r##"<a href="#" onclick="cienciaProcesso();" title="Ciência">Ciência</a>"##);
    }
    if atribuicao_observada && processo.aberto_em.iter().any(|u| u == unidade_atual) {
        write!(
            links,
            r#"<a href="controlador.php?acao=procedimento_atribuicao_cadastrar&amp;id_procedimento={id}&amp;infra_hash={assinatura}">Atribuir Processo</a>"#
        )
        .expect("formatação de String");
    }
    if relacionamentos_observados && processo.aberto_em.iter().any(|u| u == unidade_atual) {
        write!(
            links,
            r#"<a href="controlador.php?acao=procedimento_relacionar&amp;id_procedimento={id}&amp;infra_hash={assinatura}">Relacionar Processos</a>"#
        ).expect("formatação de String");
    }
    if prazo_observado && processo.aberto_em.iter().any(|u| u == unidade_atual) {
        write!(
            links,
            r#"<a href="controlador.php?acao=controle_prazo_definir&amp;id_procedimento={id}&amp;infra_hash={assinatura}">Controle de Prazo</a>"#
        )
        .expect("formatação de String");
    }
    if unidade_atual == processo.unidade_origem && processo.anotacao_origem.is_some() {
        write!(
            links,
            r#"<a href="controlador.php?acao=anotacao_registrar&amp;id_procedimento={id}&amp;infra_hash={assinatura}">Anotações</a>"#
        )
        .expect("formatação de String");
    }
    links
}
