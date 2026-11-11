//! `ies processo marcar`: prévia Web idempotente da atribuição/retirada.
//! Um comando de escrita ainda não observado nunca envia requisição modificadora.
//! A sessão, URL assinada e cookie ficam só nas peças privadas existentes.
use crate::indice_local::IndiceLocal;
use crate::marcadores_processo::MarcadoresLidos;
use crate::processo::NumeroProcesso;
use crate::processo_marcar::{
    Decisao, ErroMarcar, Operacao, Rotulo, confirmar_retirada, decidir, executar_atribuicao, recusa,
};
use crate::processo_obter::{
    EnvioDoProcesso, Marcadores, PedidoDeAndamentos, PedidoDoProcesso, SessaoDoProcesso,
    obter_processo,
};
use crate::visao_do_controle::EnvioHttps;
use crate::{Cli, Domain, Processo, ProcessoAcao, processo_concluir_cli};
use serde_json::json;

/// Despacho específico, sem tocar nos outros subcomandos.
pub(crate) fn tentar_marcar(cli: &Cli) -> Option<anyhow::Result<()>> {
    let Domain::Processo(Processo {
        acao:
            Some(ProcessoAcao::Marcar {
                numero,
                marcador,
                retirar,
                texto,
                confirmar,
            }),
        ..
    }) = &cli.command
    else {
        return None;
    };
    let nup = numero.clone();
    // O clap exige exatamente um dos dois e já interpretou o rótulo.
    let (rotulo, operacao) = match (marcador, retirar) {
        (Some(rotulo), _) => (rotulo, Operacao::Atribuir),
        (None, Some(rotulo)) => (rotulo, Operacao::Retirar),
        (None, None) => unreachable!("o clap exige --marcador ou --retirar"),
    };
    Some(processo_concluir_cli::com_sessao(cli, |sessao, indice| {
        if *confirmar
            && let Err(causa) = crate::origem_de_treinamento::exigir(
                crate::origem_de_treinamento::Escrita::ProcessoMarcar,
                sessao.origem,
            )
        {
            return Err(causa.into());
        }
        if operacao == Operacao::Atribuir {
            return executar_atribuicao(
                cli,
                sessao,
                indice,
                &nup,
                (rotulo, texto.as_deref(), *confirmar),
                &mut EnvioHttps,
            );
        }
        if *confirmar {
            return confirmar_retirada(cli, sessao, indice, &nup, rotulo, &mut EnvioHttps);
        }
        previa(
            cli,
            sessao,
            indice,
            &nup,
            (rotulo, operacao),
            *confirmar,
            &mut EnvioHttps,
        )
    }))
}

fn marcadores_observados(marcadores: &Marcadores) -> Result<&MarcadoresLidos, ErroMarcar> {
    match marcadores {
        Marcadores::Lidos(lista) => Ok(lista),
        Marcadores::Omitidos => Err(ErroMarcar::MarcadoresNaoLidos),
    }
}

fn previa<E: EnvioDoProcesso>(
    cli: &Cli,
    sessao: &SessaoDoProcesso<'_>,
    indice: &IndiceLocal,
    nup: &NumeroProcesso,
    (rotulo, operacao): (&Rotulo, Operacao),
    confirmar: bool,
    envio: &mut E,
) -> anyhow::Result<()> {
    let pedido = PedidoDoProcesso {
        nup,
        unidade: cli.unidade.as_deref(),
        andamentos: PedidoDeAndamentos::Omitidos,
        propriedades: false,
        marcadores: true,
        // Uma escrita depende da unidade publicada pela tela do processo.
        unidade_exigida: true,
        leitura_das_pastas: crate::processo_obter::LeituraDasPastas::Estrita,
    };
    let aberto = obter_processo(sessao, &pedido, indice, envio)?;
    let n = aberto.requisicoes;
    let Some(unidade) = aberto.unidade.as_deref() else {
        return Err(recusa(ErroMarcar::UnidadeDaSessaoNaoPublicada, n));
    };
    let lidos = marcadores_observados(&aberto.marcadores).map_err(|erro| recusa(erro, n))?;
    let decisao = decidir(lidos, rotulo, operacao).map_err(|erro| recusa(erro, n))?;
    if confirmar && decisao == Decisao::PreviaDeRetirada {
        // Ainda sem prova dos campos e da resposta do POST, nunca escrever.
        return Err(recusa(ErroMarcar::RemocaoSemFormularioObservado, n));
    }
    let nada_a_fazer = decisao == Decisao::NadaAFazer;
    let acao = match operacao {
        Operacao::Atribuir => "atribuir",
        Operacao::Retirar => "retirar",
    };
    if cli.json {
        println!(
            "{}",
            json!({
                "nup": nup.as_str(),
                "id_procedimento": aberto.arvore.processo.id,
                "unidade": unidade,
                "marcador": format!("{}#{}", rotulo.id, rotulo.nome),
                "acao": acao,
                "origem": "web",
                "confirmado": confirmar,
                "alterado": false,
                "nada_a_fazer": nada_a_fazer,
                "retirada_possivel": false, // Sem formulário de remoção validado, não prometer ação.
                "requisicoes": n,
            })
        );
    } else if nada_a_fazer {
        println!(
            "Processo {}: nada a fazer com {} ({}). Requisições: {}.",
            nup.as_str(),
            rotulo.id,
            acao,
            n
        );
    } else {
        println!(
            "Prévia: retirar marcador {}#{} do processo {} (unidade {}). Requisições: {}. Esta prévia não alterou o processo. Para tentar a remoção, use --confirmar em treinamento; a escrita só ocorre após formulário conferido.",
            rotulo.id,
            rotulo.nome,
            nup.as_str(),
            unidade,
            n
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn cli_exige_um_rotulo_e_nao_permite_duas_operacoes() {
        assert!(Cli::try_parse_from(["ies", "processo", "marcar", "NUP"]).is_err());
        assert!(
            Cli::try_parse_from([
                "ies",
                "processo",
                "marcar",
                "NUP",
                "--marcador",
                "1#A",
                "--retirar",
                "2#B"
            ])
            .is_err()
        );
    }

    #[test]
    fn cli_aceita_texto_so_para_atribuicao() {
        assert!(
            Cli::try_parse_from([
                "ies",
                "processo",
                "marcar",
                "NUP",
                "--marcador",
                "1#A",
                "--texto",
                "Olá"
            ])
            .is_ok()
        );
        assert!(
            Cli::try_parse_from([
                "ies",
                "processo",
                "marcar",
                "NUP",
                "--retirar",
                "1#A",
                "--texto",
                "X"
            ])
            .is_err()
        );
    }
    #[test]
    fn previa_de_retirada_e_confirmacao_bloqueada_usam_quatro_get_zero_post() {
        use crate::marcadores_processo::tests_apoio::{link_dos_marcadores, tela_dos_marcadores};
        use crate::processo_concluir::tests_apoio::{
            ID, NUP, abertura, ambiente, get, ok, pedidos_da_abertura, rodar_com,
        };
        use crate::propriedades_processo::tests_apoio::acao_da_barra;

        let url_marcadores = format!("https://sei.exemplo.gov.br/sei/{}", link_dos_marcadores(ID));
        let arvore = crate::processo_concluir::tests_apoio::arvore(
            &["TESTE"],
            &[acao_da_barra(&link_dos_marcadores(ID))],
        );
        let mut esperado = pedidos_da_abertura();
        esperado.push(get(&url_marcadores));
        for confirmar in [false, true] {
            let amb = ambiente();
            let cli =
                Cli::try_parse_from(["ies", "processo", "marcar", NUP, "--retirar", "501#Revisão"])
                    .unwrap();
            let mut respostas = abertura(arvore.clone());
            respostas.push(ok(tela_dos_marcadores(NUP, ID)));
            let (saida, pedidos) = rodar_com(&amb, respostas, |sessao, nup, indice, roteiro| {
                let rotulo = crate::processo_marcar::interpretar_rotulo("501#Revisão").unwrap();
                previa(
                    &cli,
                    sessao,
                    indice,
                    nup,
                    (&rotulo, Operacao::Retirar),
                    confirmar,
                    roteiro,
                )
            });
            if confirmar {
                assert!(matches!(
                    saida.unwrap_err().downcast_ref(),
                    Some(crate::processo_marcar::FalhaDaMarcacao::Recusada {
                        causa: ErroMarcar::RemocaoSemFormularioObservado,
                        requisicoes: 4,
                    })
                ));
            } else {
                saida.unwrap();
            }
            assert_eq!(pedidos, esperado);
            assert_eq!(pedidos.len(), 4);
            assert!(pedidos.iter().all(|pedido| pedido.metodo == "GET"));
        }
    }
}
