//! POST de salvar do Nível de detalhe (`sbmSalvar`), declarado pela própria
//! operação `unidade.definir_colunas` (contracts/cli/unidade-definir-colunas.md).
//!
//! Evidência (ensaio do proprietário na PEN1 5.0.4, treinamento, 02/11/2026):
//! o navegador envia só `hdnInfraTipoPagina` (como servido), `sbmSalvar` com
//! o valor servido e as caixas marcadas com valor `on` (sem atributo
//! `value`); a resposta é 200 sem redirect, uma página que fecha o modal,
//! e a mudança vale na leitura seguinte. Não há marcador de resultado
//! observado: o efeito é conferido por um GET da mesma tela.
use super::tela::{ErroNivelDeDetalhe, FORMULARIO_DO_NIVEL_DE_DETALHE};
use super::{ColunaDoDetalhe, PlanoDasColunas};
use crate::acao_do_sei::AcaoDoSei;
use crate::formulario_web::{
    ConferenciaDaUnidade, EfeitoAvaliado, EnvioDoFormulario, OperacaoDeFormulario,
    PreferenciaDoUsuario, ResultadoDoEnvio, localizar_formulario,
};

/// `name` das 12 caixas, na ordem de [`ColunaDoDetalhe::TODAS`].
pub(crate) const CAMPOS_DAS_OPCOES: [&str; 12] = [
    "chkSinAtribuicao",
    "chkSinAnotacao",
    "chkSinTipoProcesso",
    "chkSinEspecificacao",
    "chkSinNivelPrioritarios",
    "chkSinInteressados",
    "chkSinObservacao",
    "chkSinControlePrazo",
    "chkSinRetornoDevolver",
    "chkSinRetornoAguardando",
    "chkSinUltimaMovimentacao",
    "chkSinMarcadores",
];

/// Oculto que o formulário publica e o POST repete como servido.
pub(crate) const OCULTO_DO_NIVEL_DE_DETALHE: &str = "hdnInfraTipoPagina";
pub(crate) const BOTAO_SALVAR: &str = "sbmSalvar";

/// Salvar a tela `procedimento_configurar_detalhe`. A tela é uma janela
/// sem barra da unidade, aberta pelo link da D já conferida.
pub(crate) const SALVAR_NIVEL_DE_DETALHE: OperacaoDeFormulario = OperacaoDeFormulario {
    acao: AcaoDoSei::ProcedimentoConfigurarDetalhe,
    formulario: FORMULARIO_DO_NIVEL_DE_DETALHE,
    envio: EnvioDoFormulario::BotaoDeEnvio(BOTAO_SALVAR),
    campos_ocultos_exigidos: &[OCULTO_DO_NIVEL_DE_DETALHE],
    campos_ocultos_reescritos: &[],
    campos_usuario: &CAMPOS_DAS_OPCOES,
    resultado: ResultadoDoEnvio::ConferidoPorLeituraPosterior,
    unidade: ConferenciaDaUnidade::NaTelaDeOrigemDoLink,
    efeito: EfeitoAvaliado::AlteraPreferenciaDoUsuario {
        o_que: PreferenciaDoUsuario::ColunasDaVisaoDetalhada,
    },
};

/// Valor que o navegador envia numa caixa marcada sem atributo `value`.
pub(crate) const VALOR_DA_CAIXA_MARCADA: &str = "on";

/// Pares do usuário para o alvo: uma caixa `on` por coluna marcada.
pub(crate) fn campos_do_envio(plano: &PlanoDasColunas) -> Vec<(&'static str, &'static str)> {
    plano
        .alvo
        .iter()
        .map(|coluna| (coluna.campo(), VALOR_DA_CAIXA_MARCADA))
        .collect()
}

/// Recusa, antes de qualquer POST, controle enviável que o formulário
/// observado não tem: só `hdnInfraTipoPagina`, `sbmSalvar` e as caixas do
/// catálogo. Botões `button`/`reset` e controles `disabled` não são enviados.
pub(crate) fn conferir_controles_do_formulario(
    html: &crate::tela_html::HtmlServido,
) -> Result<(), ErroNivelDeDetalhe> {
    let dom = crate::tela_html::arvore(html);
    let form = localizar_formulario(&dom, FORMULARIO_DO_NIVEL_DE_DETALHE)
        .map_err(ErroNivelDeDetalhe::FormularioDoNivelDeDetalheAusente)?;
    for tag in form.descendentes() {
        let elemento = tag.nome().to_ascii_lowercase();
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
        let previsto = match (elemento.as_str(), tipo.as_str()) {
            // Não participam do envio.
            ("input" | "button", "button" | "reset") => true,
            ("input", "hidden") => nome == OCULTO_DO_NIVEL_DE_DETALHE,
            ("input" | "button", "submit") => nome == BOTAO_SALVAR,
            ("input", "checkbox") => ColunaDoDetalhe::pelo_campo(&nome).is_some(),
            _ => false,
        };
        if !previsto {
            return Err(ErroNivelDeDetalhe::CampoNaoPrevistoNoFormularioDoNivelDeDetalhe);
        }
    }
    Ok(())
}

/// O que a resposta HTML 200 do salvar sinaliza no `inicializar()` da
/// própria tela. Observado (capturas privadas estruturais): no GET,
/// `if (''=='1'){` (PEN1 5.0.4, GO 5.0.3, `UnB` 4.0.9); na resposta do
/// salvar que gravou, `if ('1'=='1'){` seguido de `window.parent.location`
/// e `infraFecharJanelaModal()` (PEN1 5.0.4, 02/11/2026). Só classifica a
/// resposta: o efeito continua conferido pelo GET de verificação.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SinalDoSalvar {
    /// `'1'=='1'`: o SEI sinalizou a gravação e manda fechar a janela.
    GravacaoSinalizada,
    /// `''=='1'`: a tela voltou como no GET, sem o sinal de gravação.
    TelaDevolvidaSemSinal,
    /// Nenhuma (ou mais de uma) `function inicializar(){` nessa forma.
    NaoReconhecido,
}

/// Lê o sinal por forma exata, sem executar JavaScript.
pub(crate) fn sinal_do_salvar(html: &str) -> SinalDoSalvar {
    const FUNCAO: &str = "function inicializar(){";
    let mut ocorrencias = html.match_indices(FUNCAO);
    let (Some((inicio, _)), None) = (ocorrencias.next(), ocorrencias.next()) else {
        return SinalDoSalvar::NaoReconhecido;
    };
    let Some((valor, resto)) = html
        .split_at(inicio)
        .1
        .strip_prefix(FUNCAO)
        .map(str::trim_start)
        .and_then(|resto| resto.strip_prefix("if ('"))
        .and_then(|resto| resto.split_once('\''))
    else {
        return SinalDoSalvar::NaoReconhecido;
    };
    match (valor, resto.starts_with("=='1'){")) {
        ("1", true) => SinalDoSalvar::GravacaoSinalizada,
        ("", true) => SinalDoSalvar::TelaDevolvidaSemSinal,
        _ => SinalDoSalvar::NaoReconhecido,
    }
}
