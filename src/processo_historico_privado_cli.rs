//! Consulta de andamentos em HTML privado observado, sem qualquer rede.
//! A origem, frescura, cobertura e autenticação da captura não são presumidas.
use crate::{Cli, Domain, Processo, historico, processo, transporte};
use serde_json::json;
use std::process::ExitCode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ErroHistoricoPrivado {
    NumeroAusente,
    OpcoesDeProcessoIncompativeis,
    Numero(processo::NumeroProcessoInvalido),
    Fonte(transporte::FalhaTransporte),
    Historico(historico::ErroHistorico),
}
impl ErroHistoricoPrivado {
    fn subcodigo(self) -> &'static str {
        match self {
            Self::NumeroAusente => "historico_privado_exige_nup",
            Self::OpcoesDeProcessoIncompativeis => "opcoes_de_processo_incompativeis_com_historico_privado",
            Self::Numero(erro) => erro.subcodigo(),
            Self::Fonte(erro) => erro.subcodigo(),
            Self::Historico(erro) => erro.subcodigo(),
        }
    }
    fn status(self) -> u8 {
        match self {
            Self::NumeroAusente | Self::OpcoesDeProcessoIncompativeis | Self::Numero(_) => 2,
            Self::Fonte(_) | Self::Historico(_) => 1,
        }
    }
}

fn conferir_opcoes(
    comando: &Processo,
    unidade: Option<&str>,
) -> Result<processo::NumeroProcesso, ErroHistoricoPrivado> {
    let numero = comando.numero.as_deref().ok_or(ErroHistoricoPrivado::NumeroAusente)?;
    if comando.resumo
        || comando.saida.is_some()
        || comando.caixa_local.is_some()
        || comando.observado_na_pagina
        || !comando.paginas_capturadas.is_empty()
        || comando.sem_documentos
        || comando.sem_marcadores
        || comando.sem_andamentos
        || comando.sem_propriedades
        || comando.andamentos_completos
        || comando.acao.is_some()
        || unidade.is_some()
    {
        return Err(ErroHistoricoPrivado::OpcoesDeProcessoIncompativeis);
    }
    processo::NumeroProcesso::novo(numero).map_err(ErroHistoricoPrivado::Numero)
}

fn selecionar(
    html: &str,
    numero: &processo::NumeroProcesso,
) -> Result<processo::DetalheProcesso, ErroHistoricoPrivado> {
    let _ = (html, numero);
    Err(ErroHistoricoPrivado::Historico(historico::ErroHistorico::HistoricoAusente))
}

pub(super) fn tentar_ler(cli: &Cli) -> Option<ExitCode> {
    let Domain::Processo(comando) = &cli.command else {
        return None;
    };
    let Some(arquivo) = comando.historico_privado.as_deref() else {
        return None;
    };
    let resultado = (|| {
        let numero = conferir_opcoes(comando, cli.unidade.as_deref())?;
        let html = transporte::arquivo_privado(arquivo, 8_000_000)
            .map_err(ErroHistoricoPrivado::Fonte)?;
        selecionar(&html, &numero)
    })();
    Some(match resultado {
        Ok(detalhe) => {
            if cli.json {
                println!("{}", json!({
                    "processo": detalhe, "escopo": "historico_privado_offline",
                    "parcial": true, "autenticacao_remota": false
                }));
            } else {
                println!(
                    "IES: {} andamento(s) observados no HTML privado do processo {}; histórico remoto não certificado.",
                    detalhe.andamentos.len(),
                    detalhe.numero.as_str()
                );
            }
            ExitCode::SUCCESS
        }
        Err(erro) => {
            if cli.json {
                println!("{}", json!({
                    "codigo": if erro.status() == 2 { "entrada_invalida" } else { "fonte_invalida" },
                    "subcodigo": erro.subcodigo(),
                    "erro": "histórico privado de processo não comprovado",
                    "retentavel": false
                }));
            } else {
                eprintln!("IES: histórico privado recusado [{}]", erro.subcodigo());
            }
            ExitCode::from(erro.status())
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    fn historico_html(numero: &str) -> String {
        format!(
            "<html><head><title>SEI - Histórico do Processo {numero}</title></head><body>
             <table id='tblHistorico'><tr><th>Data/Hora</th></tr>
             <tr><td>31/10/2026</td><td>UNIDADE A</td><td>usuario</td>
             <td>Encaminhado</td></tr></table></body></html>"
        )
    }

    #[test]
    fn consulta_historico_capturado_e_preserva_nup_e_andamentos() {
        let numero = processo::NumeroProcesso::novo("000123/2026").unwrap();
        let detalhe = selecionar(&historico_html("000123/2026"), &numero).unwrap();
        assert_eq!(detalhe.numero.as_str(), "000123/2026");
        assert_eq!(detalhe.andamentos.len(), 1);
        assert_eq!(detalhe.andamentos[0].descricao, "Encaminhado");
    }

    #[test]
    fn recusa_outro_nup_e_tabelas_de_historico_duplicadas() {
        let numero = processo::NumeroProcesso::novo("000123/2026").unwrap();
        assert_eq!(
            selecionar(&historico_html("000124/2026"), &numero),
            Err(ErroHistoricoPrivado::Historico(
                historico::ErroHistorico::NumeroDoHistoricoDiferenteDoSolicitado
            ))
        );
        let duplicado = historico_html("000123/2026").replace(
            "</body>",
            "<table id='tblHistorico'></table></body>"
        );
        assert_eq!(
            selecionar(&duplicado, &numero),
            Err(ErroHistoricoPrivado::Historico(
                historico::ErroHistorico::TabelasDoHistoricoRepetidas
            ))
        );
    }

    #[test]
    fn cli_aceita_opt_in_e_recusa_mistura_de_fonte_com_caixa() {
        let cli = Cli::try_parse_from([
            "ies", "processo", "000123/2026", "--historico-privado", "/tmp/historico.html"
        ]).unwrap();
        let Domain::Processo(comando) = cli.command else { panic!("não é processo") };
        assert!(conferir_opcoes(&comando, None).is_ok());

        let cli = Cli::try_parse_from([
            "ies", "processo", "000123/2026", "--resumo",
            "--historico-privado", "/tmp/historico.html"
        ]).unwrap();
        let Domain::Processo(comando) = cli.command else { panic!("não é processo") };
        assert_eq!(
            conferir_opcoes(&comando, None),
            Err(ErroHistoricoPrivado::OpcoesDeProcessoIncompativeis)
        );
    }

    #[test]
    fn nup_com_controle_tem_variante_propria_antes_de_ler_html() {
        let cli = Cli::try_parse_from([
            "ies", "processo", "000123/2026\n", "--historico-privado", "/tmp/historico.html"
        ]).unwrap();
        let Domain::Processo(comando) = cli.command else { panic!("não é processo") };
        assert_eq!(
            conferir_opcoes(&comando, None),
            Err(ErroHistoricoPrivado::Numero(processo::NumeroProcessoInvalido::Controle))
        );
    }
}
