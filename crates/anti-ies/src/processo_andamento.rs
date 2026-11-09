//! Histórico e prévia de registro de andamento como projeções do SQLite.
//! Somente leitura: o --confirmar permanece fechado na CLI pública.
use crate::{Estado, Resposta, documentos::campo_formulario_unico, escapar, parametro_unico};
use iesei::andamentos::{self, Historico, Linha};
use iesei::banco_cassettes;
use iesei::instalacao::Instalacao;
use iesei::processos;
use rusqlite::Connection;
use std::fmt::Write as _;

fn pagina_historico(numero: &str, historico: &Historico, completo: bool) -> Resposta {
    let total = historico.total_declarado;
    let palavra = if total == 1 { "registro" } else { "registros" };
    let tipo = if completo { "P" } else { "R" };
    let mut html = format!(
        "<html><head><title>SEI - Histórico do Processo {}</title></head><body>\
         <form id='frmProcedimentoHistorico' method='post' \
         action='controlador.php?acao=procedimento_consultar_historico'>\
         <input type='hidden' id='hdnTipoHistorico' name='hdnTipoHistorico' value='{tipo}'>\
         <table id='tblHistorico' class='infraTable' summary='Histórico de Andamentos'>\
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
    html.push_str(
        "</table></form><a id='ancTipoHistorico' onclick=\"verHistorico('P');\">Ver histórico completo</a>\n\
         <script>\n\
         function verHistorico(valor){\n\
         document.getElementById('hdnTipoHistorico').value = valor;\n\
         document.getElementById('frmProcedimentoHistorico').submit();\n\
         }\n\
         </script></body></html>",
    );
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
    match (parametro_unico(alvo, "acao"), metodo) {
        (Some("procedimento_consultar_historico"), "GET") => {
            pagina_historico(&processo.numero, &historico, false)
        }
        (Some("procedimento_consultar_historico"), "POST") => {
            if corpo.split('&').count() != 1
                || campo_formulario_unico(corpo, "hdnTipoHistorico").as_deref() != Some("P")
            {
                return Resposta::nova(Estado::RequisicaoInvalida);
            }
            pagina_historico(&processo.numero, &historico, true)
        }
        (Some("procedimento_atualizar_andamento"), "GET") => pagina_formulario(id),
        (Some("procedimento_atualizar_andamento"), "POST") => {
            registrar_sintetico(corpo, banco, instalacao, cenario, id, &processo)
        }
        (Some("procedimento_consultar_historico" | "procedimento_atualizar_andamento"), _) => {
            Resposta::nova(Estado::MetodoNaoPermitido)
        }
        _ => Resposta::nova(Estado::NaoImplementado),
    }
}

/// Hipótese de transição local: os dados de formulário devem corresponder
/// à tela oferecida e ao processo aberto. Nenhuma escrita sai do SQLite.
fn registrar_sintetico(
    corpo: &str,
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
    id: &str,
    processo: &processos::ProcessoSimulado,
) -> Resposta {
    let (Some(protocolo), Some(atividade), Some(texto), Some(botao)) = (
        campo_formulario_unico(corpo, "hdnIdProtocolo"),
        campo_formulario_unico(corpo, "hdnIdAtividade"),
        campo_formulario_unico(corpo, "txaDescricao"),
        campo_formulario_unico(corpo, "sbmSalvar"),
    ) else {
        return Resposta::nova(Estado::RequisicaoInvalida);
    };
    if corpo.split('&').count() != 4
        || protocolo != id
        || !atividade.is_empty()
        || botao != "Salvar"
        || texto.trim().is_empty()
        || texto.chars().count() > 500
    {
        return Resposta::nova(Estado::RequisicaoInvalida);
    }
    let Ok(Some(controle)) = banco_cassettes::ler(banco, instalacao, cenario) else {
        return Resposta::nova(Estado::DadosInsuficientes);
    };
    let unidade = controle.unidade.as_str();
    if !processo.aberto_em.iter().any(|aberta| aberta == unidade) {
        return Resposta::nova(Estado::RequisicaoInvalida);
    }
    // Relógio/usuário não foram observados nesta transição: valores
    // explicitamente sintéticos, sem reivindicar proveniência de cassette.
    let linha = Linha {
        ocorrido_em: "09/11/2026 11:00".into(),
        unidade: unidade.to_owned(),
        usuario: "simulador".into(),
        descricao: texto,
    };
    match andamentos::acrescentar(banco, instalacao, cenario, id, linha) {
        Ok(()) => Resposta::redirecionar(format!(
            "controlador.php?acao=procedimento_consultar_historico&id_procedimento={id}"
        )),
        Err(andamentos::Falha::DadoInvalido) => Resposta::nova(Estado::RequisicaoInvalida),
        Err(andamentos::Falha::Sqlite(_)) => Resposta::nova(Estado::DadosInsuficientes),
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
        let html = pagina_historico("00001.000001/2026-01", &h, false).corpo;
        assert!(html.contains("Anterior"));
        assert!(html.contains("Lista de Andamentos (1 registro):"));
        assert!(!html.contains("Registro novo"));
    }
}
