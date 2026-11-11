//! Diagnóstico opt-in do Controle Web: nunca revela processos ou links assinados.
use crate::erro_web::ErroCaixa;
use crate::{configuracao, transporte};
use serde_json::{Value, json};

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum FalhaControleWeb {
    ControleNaoReconhecido(ErroCaixa),
    IdentificacaoDaUnidadeAusenteNoControle,
    UnidadeExibidaDiferenteDaSolicitada,
}
/// A tela precisa conter Controle de Processos e unidade única, não apenas HTTP 200.
pub(crate) fn projetar(html: &str) -> Result<Value, FalhaControleWeb> {
    // Visão resumida ou detalhada: a sessão fica na última visão usada.
    let unidade =
        crate::controle_reconhecido::unidade_do_controle(html).map_err(|erro| match erro {
            ErroCaixa::IdentificacaoDaUnidadeAusenteNoControle => {
                FalhaControleWeb::IdentificacaoDaUnidadeAusenteNoControle
            }
            outro @ (ErroCaixa::TabelaAusente { .. }
            | ErroCaixa::TabelasDaCategoriaRepetidas { .. }
            | ErroCaixa::TabelaInvalida { .. }
            | ErroCaixa::LegendaAusente { .. }
            | ErroCaixa::LegendasDaTabelaRepetidas { .. }
            | ErroCaixa::IdInvalido
            | ErroCaixa::IdRepetido { .. }
            | ErroCaixa::NumeroInvalido { .. }
            | ErroCaixa::MaisDeUmNumeroNaLinhaDoProcesso { .. }
            | ErroCaixa::LinkDoProcessoNaoIdentificadoNaLinha { .. }
            | ErroCaixa::ClassesDeEstadoDiferentesNoLinkDoProcesso { .. }
            | ErroCaixa::TotalExcedido { .. }
            | ErroCaixa::TotaisDeclaradosDiferentesEntrePaginas { .. }
            | ErroCaixa::FaixaInvalida { .. }
            | ErroCaixa::FaixaDesconhecida { .. }
            | ErroCaixa::FaixaAlterada { .. }
            | ErroCaixa::FaixaDescontinua { .. }
            | ErroCaixa::TotalNaoDeclaradoNaLegenda { .. }
            | ErroCaixa::LegendaSemFaixaComTotalDiferenteDasLinhas { .. }
            | ErroCaixa::PaginadorDaTabelaSemFaixaNaLegenda { .. }
            | ErroCaixa::LimitePaginas
            | ErroCaixa::VisaoResumidaComPaginacao
            | ErroCaixa::FiltroDeMarcadorAtivoNoControle
            | ErroCaixa::ContextoNaoVerificavel
            | ErroCaixa::ContextoDaPaginaDiferenteDoAnterior
            | ErroCaixa::SemPaginas
            | ErroCaixa::HashesDeCriteriosGeradosRepetidosNoControle
            | ErroCaixa::HashesDeCriteriosRecebidosRepetidosNoControle
            | ErroCaixa::HtmlDaCaixaInvalido
            | ErroCaixa::HtmlDaCaixaRecusadoPeloParser(_)
            | ErroCaixa::AtributoDaCaixaIlegivel(_)
            | ErroCaixa::IdentificacoesDaUnidadeRepetidasNoControle
            | ErroCaixa::TiposDeVisualizacaoRepetidosNoControle
            | ErroCaixa::UnidadeInesperada) => FalhaControleWeb::ControleNaoReconhecido(outro),
        })?;
    Ok(json!({
        "controle_web": "observado",
        "unidade_exibida": unidade,
        "conexao": "get_realizado",
        "autenticacao_remota": "nao_auditada",
    }))
}

/// Shim de conferência: o comportamento será guiado por testes do contrato.
#[expect(
    clippy::indexing_slicing,
    reason = "serde_json::Value: leitura ausente dá Null e a escrita é em objeto json! local; projeção de saída a substituir em #733"
)]
fn projetar_com_unidade(
    html: &str,
    unidade_esperada: Option<&str>,
) -> Result<Value, FalhaControleWeb> {
    let modelo = projetar(html)?;
    if unidade_esperada.is_some_and(|esperada| modelo["unidade_exibida"].as_str() != Some(esperada))
    {
        return Err(FalhaControleWeb::UnidadeExibidaDiferenteDaSolicitada);
    }
    Ok(modelo)
}

/// Somente com pedido explícito: um GET da sessão já configurada, sem novo login.
#[expect(
    clippy::indexing_slicing,
    reason = "serde_json::Value: leitura ausente dá Null e a escrita é em objeto json! local; projeção de saída a substituir em #733"
)]
pub(crate) fn mostrar(json_mode: bool, unidade_esperada: Option<&str>) -> anyhow::Result<()> {
    let arquivos = configuracao::arquivos_de_acesso()?;
    let indice = crate::indice_local::IndiceLocal::da_configuracao(&arquivos);
    // Um retrato coerente da sessão (#222) para o GET e para a colheita.
    let sessao = transporte::sessao::ReferenciasDaSessao::ler(
        &arquivos.url_controle,
        &arquivos.origem,
        &arquivos.cookie,
    )?;
    let tela = transporte::envio_post::ler_tela_do_controle(&sessao)?;
    crate::colheita_do_controle::Colheita::das_referencias(&indice, &sessao)
        .colher(&tela.html, Some(&tela.mime));
    let mut modelo = projetar_com_unidade(&tela.html, unidade_esperada)?;
    if json_mode {
        crate::indice_local::anexar_avisos(&mut modelo, &indice);
        println!("{modelo}");
    } else {
        crate::indice_local::avisar_no_texto(&indice);
        println!(
            "IES: Controle Web observado na unidade {}; autenticação não auditada.",
            modelo["unidade_exibida"].as_str().unwrap_or("desconhecida")
        );
    }
    Ok(())
}

#[cfg(test)]
mod testes {
    use super::*;
    use clap::Parser;

    #[test]
    fn comando_web_requer_opt_in_e_recusa_mistura_com_validacao_local() {
        assert!(crate::Cli::try_parse_from(["ies", "status", "--verificar-controle-web"]).is_ok());
        assert!(
            crate::Cli::try_parse_from([
                "ies",
                "status",
                "--verificar-controle-web",
                "--verificar-sessao-local"
            ])
            .is_err()
        );
        assert!(
            crate::Cli::try_parse_from([
                "ies",
                "status",
                "--verificar-controle-web",
                "--verificar-arquivos"
            ])
            .is_err()
        );
    }

    #[test]
    fn pagina_de_controle_identificada_sem_expor_conteudo_da_caixa() {
        let html = "<a id='lnkInfraUnidade'>PPI</a>\
                    <table id='tblProcessosRecebidos'><caption>0 registros:</caption></table>\
                    <table id='tblProcessosGerados'><caption>0 registros:</caption></table>";
        let valor = projetar(html).unwrap();
        assert_eq!(valor["controle_web"], "observado");
        assert_eq!(valor["unidade_exibida"], "PPI");
        assert_eq!(valor["autenticacao_remota"], "nao_auditada");
        assert_eq!(valor["conexao"], "get_realizado");
        assert!(valor.get("processos").is_none());
        assert!(!valor.to_string().contains("infra_hash"));
    }

    #[test]
    fn formulario_de_login_nao_prova_controle_web() {
        // HTTP 200 informa sobre o transporte, não identifica a página do SEI.
        let html = "<html><body><form id='frmLogin' method='post'>\
                    <input name='txtUsuario' type='text'>\
                    <input name='pwdSenha' type='password'>\
                    </form></body></html>";
        assert!(
            projetar(html).is_err(),
            "formulário de login não pode ser tratado como Controle Web"
        );
    }

    #[test]
    fn unidade_global_errada_nao_pode_gerar_sucesso_web() {
        let html = "<a id='lnkInfraUnidade'>PPI</a>\
                    <table id='tblProcessosRecebidos'><caption>0 registros:</caption></table>\
                    <table id='tblProcessosGerados'><caption>0 registros:</caption></table>";
        assert_eq!(
            projetar_com_unidade(html, Some("OUTRA")),
            Err(FalhaControleWeb::UnidadeExibidaDiferenteDaSolicitada)
        );
        assert_eq!(
            projetar_com_unidade(html, Some("ppi")),
            Err(FalhaControleWeb::UnidadeExibidaDiferenteDaSolicitada)
        );
    }

    #[test]
    fn unidade_global_igual_ou_ausente_preserva_diagnostico() {
        let html = "<a id='lnkInfraUnidade'>PPI</a>\
                    <table id='tblProcessosRecebidos'><caption>0 registros:</caption></table>\
                    <table id='tblProcessosGerados'><caption>0 registros:</caption></table>";
        for esperada in [None, Some("PPI")] {
            let valor = projetar_com_unidade(html, esperada).unwrap();
            assert_eq!(valor["unidade_exibida"], "PPI");
            assert_eq!(valor["autenticacao_remota"], "nao_auditada");
        }
    }

    #[test]
    fn controles_sem_rotulo_de_unidade_nao_provam_contexto() {
        let html = "<table id='tblProcessosRecebidos'><caption>0 registros:</caption></table>\
                    <table id='tblProcessosGerados'><caption>0 registros:</caption></table>";
        assert_eq!(
            projetar(html),
            Err(FalhaControleWeb::IdentificacaoDaUnidadeAusenteNoControle)
        );
    }
}
