//! Histórico e prévia de registro de andamento como projeções do SQLite.
//! Escrita sintética limitada ao backend local: sem rede externa.
use crate::prazo_processo::campo_unico;
use crate::{Estado, Resposta, escapar, parametro_unico};
use iesei::andamentos::{self, Historico, Linha};
use iesei::instalacao::Instalacao;
use iesei::processos;
use rusqlite::Connection;
use std::fmt::Write as _;

fn pagina_historico(numero: &str, historico: &Historico) -> Resposta {
    let total = historico.total_declarado;
    let palavra = if total == 1 { "registro" } else { "registros" };
    let mut html = format!(
        "<html><head><title>SEI - Histórico do Processo {}</title></head><body>\
         <form id='frmProcedimentoHistorico' method='post' \
         action='controlador.php?acao=procedimento_consultar_historico'>\
         <input type='hidden' name='hdnTipoHistorico' value='R'>\
         <table id='tblHistorico' summary='Histórico de Andamentos'>\
         <caption>Lista de Andamentos ({total} {palavra}):</caption>\
         <tr><th>Data/Hora</th><th>Unidade</th><th>Usuário</th><th>Descrição</th></tr>",
        escapar(numero)
    );
    for linha in &historico.linhas {
        write!(
            html,
            "<tr class='andamentoConcluido'><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
            escapar(&linha.ocorrido_em),
            escapar(&linha.unidade),
            escapar(&linha.usuario),
            escapar(&linha.descricao),
        )
        .expect("formatação String");
    }
    html.push_str("</table></form></body></html>");
    Resposta::html(html)
}

fn pagina_formulario(id: &str) -> Resposta {
    let hash = crate::protocolo_sintetico::INFRA_HASH;
    Resposta::html(format!(
        "<html><body><form id='frmAtividadeListar' method='post' \
         action='controlador.php?acao=procedimento_atualizar_andamento&amp;id_procedimento={id}&amp;infra_hash={hash}'>\
         <input type='hidden' name='hdnIdProtocolo' value='{id}'>\
         <input type='hidden' name='hdnIdAtividade' value=''>\
         <textarea name='txaDescricao'></textarea>\
         <input type='submit' name='sbmSalvar' value='Salvar'>\
         </form></body></html>"
    ))
}

/// O POST usa somente campos derivados do formulário e persiste a nova
/// linha no SQLite efêmero; horário e autoria são hipótese sintética.
fn gravar(
    corpo: &str,
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
    id: &str,
    historico: &Historico,
) -> Resposta {
    let (Some(processo), Some(atividade), Some(texto), Some(botao)) = (
        campo_unico(corpo, "hdnIdProtocolo"),
        campo_unico(corpo, "hdnIdAtividade"),
        campo_unico(corpo, "txaDescricao"),
        campo_unico(corpo, "sbmSalvar"),
    ) else {
        return Resposta::nova(Estado::RequisicaoInvalida);
    };
    if corpo.split('&').count() != 4
        || processo != id
        || !atividade.is_empty()
        || botao != "Salvar"
        || texto.trim().is_empty()
        || texto.chars().count() > 500
    {
        return Resposta::nova(Estado::RequisicaoInvalida);
    }
    let Some(anterior) = historico.linhas.first() else {
        return Resposta::nova(Estado::DadosInsuficientes);
    };
    let nova = Linha {
        ocorrido_em: "09/11/2026 10:01".to_owned(),
        unidade: anterior.unidade.clone(),
        usuario: anterior.usuario.clone(),
        descricao: texto,
    };
    match andamentos::acrescentar(banco, instalacao, cenario, id, nova) {
        Ok(()) => Resposta::redirecionar(format!(
            "controlador.php?acao=procedimento_consultar_historico&id_procedimento={id}"
        )),
        Err(_) => Resposta::nova(Estado::DadosInsuficientes),
    }
}

/// Serve formulário e histórico já registrados, sem fingir dados ausentes.
pub(crate) fn responder(
    metodo: &str,
    alvo: &str,
    corpo: &str,
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
) -> Resposta {
    if !matches!(metodo, "GET" | "POST") {
        return Resposta::nova(Estado::MetodoNaoPermitido);
    }
    let Some(id) = parametro_unico(alvo, "id_procedimento")
        .filter(|id| id.bytes().all(|b| b.is_ascii_digit()) && !id.is_empty())
    else {
        return Resposta::nova(Estado::RequisicaoInvalida);
    };
    let Ok(Some(historico)) = andamentos::ler(banco, instalacao, cenario, id) else {
        return Resposta::nova(Estado::DadosInsuficientes);
    };
    let Ok(Some(processo)) = processos::ler(banco, instalacao, cenario, id) else {
        return Resposta::nova(Estado::DadosInsuficientes);
    };
    match (metodo, parametro_unico(alvo, "acao")) {
        ("GET", Some("procedimento_consultar_historico")) => {
            pagina_historico(&processo.numero, &historico)
        }
        ("GET", Some("procedimento_atualizar_andamento")) => pagina_formulario(id),
        ("POST", Some("procedimento_atualizar_andamento")) => {
            gravar(corpo, banco, instalacao, cenario, id, &historico)
        }
        ("POST", Some("procedimento_consultar_historico")) => {
            Resposta::nova(Estado::MetodoNaoPermitido)
        }
        _ => Resposta::nova(Estado::NaoImplementado),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iesei::andamentos::Linha;

    #[test]
    fn historico_publica_linhas_sem_inventar_uma_nova() {
        let h = Historico {
            total_declarado: 1,
            linhas: vec![Linha {
                ocorrido_em: "09/11/2026 10:00".into(),
                unidade: "TESTE".into(),
                usuario: "aluno".into(),
                descricao: "Anterior".into(),
            }],
        };
        let html = pagina_historico("00001.000001/2026-01", &h).corpo;
        assert!(html.contains("Anterior"));
        assert!(html.contains("Lista de Andamentos (1 registro):"));
        assert!(!html.contains("Registro novo"));
    }
}
