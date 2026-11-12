//! A tela "Conclusão de Processo" (forma `tela`, SEI 4.1 e 5.0) e a
//! resposta das duas formas (contracts/cli/processo-concluir.md).
//!
//! Evidência (cassettes públicas do Todos, UFRJ 4.1.5, PEN2 4.1.5, GO
//! 5.0.3, PEN3 e PEN4 5.0.4, telas idênticas): `form#frmDesentranharDocumento`
//! `method="post"`; ocultos `hdnInfraTipoPagina`, `hdnIdProtocolos` (o id) e
//! `hdnIdProcedimentoDestino`; `sbmSalvar`; `rdoConcluir` `S` marcado e `V`;
//! a reabertura programada vazia. O navegador envia `rdoConcluir=S` e os
//! dois textos vazios (teste do Todos que mediu o envio na UFRJ); o SEI
//! responde 302 para `arvore_visualizar` (ou `principal`), da conclusão do
//! mesmo processo.
use super::erro::ErroTelaDeConclusao;
use crate::acao_do_sei::AcaoDoSei;
use crate::consulta_do_controlador::ConsultaDoControlador;
use crate::formulario_web::{
    ConferenciaDaUnidade, EfeitoAvaliado, EnvioDoFormulario, EstadoDoProcesso,
    OperacaoDeFormulario, ResultadoDoEnvio, consulta_do_controlador, localizar_formulario,
};

pub(crate) const FORMULARIO_DE_CONCLUSAO: &str = "frmDesentranharDocumento";
pub(crate) const BOTAO_SALVAR: &str = "sbmSalvar";
const OCULTOS: [&str; 3] = [
    "hdnInfraTipoPagina",
    "hdnIdProtocolos",
    "hdnIdProcedimentoDestino",
];
const OPCAO: &str = "rdoConcluir";
const SOMENTE_CONCLUIR: &str = "S";
const PRAZO: &str = "rdoPrazoReaberturaProgramada";
const DATA: &str = "txtPrazoReaberturaProgramada";
const DIAS: &str = "txtDiasReaberturaProgramada";
const UTEIS: &str = "chkSinDiasUteisReaberturaProgramada";
const PROCESSOS: &str = "selProcedimentos";

/// Salvar a tela de conclusão: janela sem barra da unidade, aberta pelo
/// link da barra da árvore da tela do processo conferida.
pub(crate) const CONCLUIR_PROCESSO: OperacaoDeFormulario = OperacaoDeFormulario {
    acao: AcaoDoSei::ProcedimentoConcluir,
    formulario: FORMULARIO_DE_CONCLUSAO,
    envio: EnvioDoFormulario::BotaoDeEnvio(BOTAO_SALVAR),
    campos_ocultos_exigidos: &OCULTOS,
    campos_ocultos_reescritos: &[],
    campos_usuario: &[OPCAO, DATA, DIAS],
    resultado: ResultadoDoEnvio::ConferidoPorLeituraPosterior,
    unidade: ConferenciaDaUnidade::NaTelaDeOrigemDoLink,
    efeito: EfeitoAvaliado::AlteraEstadoDoProcesso {
        o_que: EstadoDoProcesso::AndamentoAbertoNaUnidade,
    },
};

/// Os pares do usuário: "Somente concluir" e os textos vazios.
pub(crate) const CAMPOS_DO_ENVIO: [(&str, &str); 3] =
    [(OPCAO, SOMENTE_CONCLUIR), (DATA, ""), (DIAS, "")];

/// Confere a tela servida antes de qualquer POST: lista fechada de
/// controles enviáveis, o processo, "Somente concluir" marcado e nada da
/// reabertura programada preenchido.
pub(crate) fn conferir_tela_de_conclusao(
    html: &crate::tela_html::HtmlServido,
    id: &str,
) -> Result<(), ErroTelaDeConclusao> {
    let dom = crate::tela_html::arvore(html);
    let form = localizar_formulario(&dom, FORMULARIO_DE_CONCLUSAO)
        .map_err(ErroTelaDeConclusao::FormularioDeConclusaoAusente)?;
    let mut somente_marcado = false;
    let mut outra_marcada = false;
    let mut preenchida = false;
    for tag in form.descendentes() {
        let elemento = tag.nome().to_ascii_lowercase();
        if elemento == "option" {
            preenchida |= tag.presente("selected");
            continue;
        }
        if !matches!(
            elemento.as_str(),
            "input" | "button" | "select" | "textarea"
        ) || tag.presente("disabled")
        {
            continue;
        }
        let Some(nome) = tag
            .atributo("name")
            .map(str::to_owned)
            .filter(|n| !n.is_empty())
        else {
            continue;
        };
        let padrao = if elemento == "button" {
            "submit"
        } else {
            "text"
        };
        let tipo = tag
            .atributo("type")
            .map(str::to_owned)
            .map_or_else(|| padrao.to_owned(), |t| t.to_ascii_lowercase());
        let marcado = tag.presente("checked");
        let servido = tag.atributo("value").map(str::to_owned).unwrap_or_default();
        match (elemento.as_str(), tipo.as_str(), nome.as_str()) {
            ("input", "hidden", "hdnIdProtocolos") if servido != id => {
                return Err(ErroTelaDeConclusao::TelaDeConclusaoDeOutroProcesso);
            }
            ("input", "hidden", n) if OCULTOS.contains(&n) => {}
            // `button`/`reset` não participam do envio.
            ("input" | "button", "button" | "reset", _)
            | ("input" | "button", "submit", BOTAO_SALVAR)
            | ("select", _, PROCESSOS) => {}
            ("input", "radio", OPCAO) => {
                if marcado && servido == SOMENTE_CONCLUIR {
                    somente_marcado = true;
                } else if marcado {
                    outra_marcada = true;
                }
            }
            ("input", "radio", PRAZO) | ("input", "checkbox", UTEIS) => preenchida |= marcado,
            ("input", "text", DATA | DIAS) => preenchida |= !servido.is_empty(),
            _ => return Err(ErroTelaDeConclusao::CampoNaoPrevistoNaTelaDeConclusao),
        }
    }
    if !somente_marcado || outra_marcada {
        return Err(ErroTelaDeConclusao::OpcaoSomenteConcluirNaoMarcada);
    }
    if preenchida {
        return Err(ErroTelaDeConclusao::TelaDeConclusaoComReaberturaPreenchida);
    }
    Ok(())
}

/// O 302 do POST é o esperado: `controlador.php?acao=arvore_visualizar`
/// ou `acao=principal` (relativo, como gravado, ou no host e na pasta do
/// POST: ver `consulta_do_controlador`), com
/// `acao_origem=procedimento_concluir` e o id, cada chave uma vez.
pub(crate) fn redirecionamento_esperado(
    localizacao: Option<&str>,
    url_do_envio: &str,
    id: &str,
) -> bool {
    let Some(consulta) = localizacao.and_then(|l| consulta_do_controlador(l, url_do_envio)) else {
        return false;
    };
    ConsultaDoControlador::ler(consulta).is_ok_and(|consulta| {
        matches!(
            consulta.acao(),
            Ok(AcaoDoSei::ArvoreVisualizar | AcaoDoSei::Principal)
        ) && consulta.acao_de_origem() == Ok(AcaoDoSei::ProcedimentoConcluir)
            && consulta.unico("id_procedimento") == Ok(id)
    })
}
