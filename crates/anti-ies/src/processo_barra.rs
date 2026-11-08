//! Ações da barra do processo, derivadas da unidade e anotação no SQLite.
//! Links assinados aqui são estritamente sintéticos; não há credencial.
use iesei::processos::ProcessoSimulado;
use std::fmt::Write as _;

/// Renderiza apenas ações oferecidas pelo estado conhecido da origem.
pub(crate) fn acoes_do_processo(processo: &ProcessoSimulado) -> String {
    let mut links = String::new();
    let id = &processo.id;
    let assinatura = "4".repeat(64);
    if processo.aberto_em.contains(&processo.unidade_origem) {
        write!(
            links,
            r#"<a href="controlador.php?acao=procedimento_enviar&amp;acao_origem=arvore_visualizar&amp;id_procedimento={id}&amp;arvore=1&amp;infra_hash={assinatura}">Enviar</a>"#
        )
        .expect("formatação de String");
    }
    if processo.anotacao_origem.is_some() {
        write!(
            links,
            r#"<a href="controlador.php?acao=anotacao_registrar&amp;id_procedimento={id}&amp;infra_hash={assinatura}">Anotações</a>"#
        )
        .expect("formatação de String");
    }
    links
}
