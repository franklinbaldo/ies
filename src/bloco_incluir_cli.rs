//! CLI de `ies bloco incluir BLOCO --documento DOC | --processo NUP [--confirmar]`
//! (#381; contracts/cli/bloco-incluir.md). BDD-381-01/02: prévia e confirmação
//! causal documental e BDD-381-04: prévia processual sem escrita.
//! Erros sobem até o `main` (#695).
use crate::bloco::TipoBloco;
use crate::bloco_cli::{Leitura, ler_bloco_por_numero_via};
use crate::bloco_incluir::{PedidoDaInclusao, PreviaDocumental, prever_documento};
use crate::documento::NumeroDocumento;
use crate::indice_local::{self, IndiceLocal};
use crate::processo::NumeroProcesso;
use crate::processo_concluir::escrita::{Contador, leitura};
use crate::processo_obter::SessaoDoProcesso;
use crate::processo_obter::obter_processo;
use crate::transporte::sessao::ReferenciasDaSessao;
use crate::visao_do_controle::EnvioHttps;
use crate::{Cli, configuracao};
use serde_json::json;

/// O que esta fatia ainda não faz: recusado antes de qualquer requisição.
#[derive(Debug, thiserror::Error)]
#[error("{self:?}")]
enum AindaNaoImplementado {
    InclusaoDeProcesso,
}

pub(crate) fn executar(
    cli: &Cli,
    bloco: &str,
    documento: Option<&NumeroDocumento>,
    processo: Option<&str>,
    confirmar: bool,
) -> anyhow::Result<()> {
    if let Some(nup) = processo {
        if confirmar {
            return Err(AindaNaoImplementado::InclusaoDeProcesso.into());
        }
        return prever_processo(cli, bloco, nup);
    }
    let documento = documento.ok_or(AindaNaoImplementado::InclusaoDeProcesso)?;
    let arquivos = configuracao::arquivos_de_acesso()?;
    let referencias =
        ReferenciasDaSessao::ler(&arquivos.url_controle, &arquivos.origem, &arquivos.cookie)?;
    if confirmar {
        crate::origem_de_treinamento::exigir(
            crate::origem_de_treinamento::Escrita::BlocoIncluirDocumento,
            referencias.origem(),
        )?;
    }
    let sessao = SessaoDoProcesso {
        url_do_controle: referencias.url_do_controle(),
        origem: referencias.origem(),
        cookie: referencias.cookie(),
    };
    let indice = IndiceLocal::da_configuracao(&arquivos);
    let previa = prever_documento(
        &sessao,
        &referencias,
        cli.unidade.as_deref(),
        &PedidoDaInclusao {
            bloco,
            documento,
            confirmar,
        },
        &indice,
        &mut EnvioHttps,
    )?;
    publicar(cli.json, &previa, &indice);
    Ok(())
}

fn publicar(json_mode: bool, previa: &PreviaDocumental, indice: &IndiceLocal) {
    if json_mode {
        let mut valor = json!({
                "bloco": previa.bloco,
                "tipo_bloco": "assinatura",
                "alvo": {"documento": previa.documento},
                "confirmado": previa.desfecho.confirmado(),
                "alterado": previa.desfecho.alterado(),
                "presente_antes": previa.desfecho.presente_antes(),
                "presente_depois": previa.desfecho.presente_depois(),
                "origem": "web",
                "requisicoes": previa.requisicoes,
        });
        indice_local::anexar_avisos(&mut valor, indice);
        println!("{valor}");
        return;
    }
    indice_local::avisar_no_texto(indice);
    if previa.desfecho.alterado() {
        println!(
            "Documento {} incluído e conferido no bloco de assinatura {}. Requisições: {}",
            previa.documento, previa.bloco, previa.requisicoes
        );
    } else if previa.desfecho.presente_antes() {
        println!(
            "Documento {} já está no bloco de assinatura {}; nenhuma alteração necessária. Requisições: {}",
            previa.documento, previa.bloco, previa.requisicoes
        );
    } else {
        let antes = if previa.desfecho.presente_antes() {
            "já está no bloco; nada a incluir"
        } else {
            "não está no bloco"
        };
        println!(
            "Prévia: documento {} no bloco de assinatura {}: {antes}. Para incluir, repita com --confirmar. Requisições: {}",
            previa.documento, previa.bloco, previa.requisicoes
        );
    }
}

/// Causas tipadas de prévia processual, sem novo catálogo de subcódigos.
#[derive(Debug, thiserror::Error)]
#[error("{self:?}")]
enum ErroPreviaProcessual {
    TipoInadequado,
    PertencimentoNaoComprovado,
    IdentidadeDoProcessoDivergente,
}

/// Primeira fatia do processo: apenas leitura do bloco e abertura do NUP.
/// A confirmação permanece bloqueada até BDD-381-05.
fn prever_processo(cli: &Cli, bloco: &str, nup_bruto: &str) -> anyhow::Result<()> {
    let nup = NumeroProcesso::novo(nup_bruto)?;
    let arquivos = configuracao::arquivos_de_acesso()?;
    let referencias =
        ReferenciasDaSessao::ler(&arquivos.url_controle, &arquivos.origem, &arquivos.cookie)?;
    let sessao = SessaoDoProcesso {
        url_do_controle: referencias.url_do_controle(),
        origem: referencias.origem(),
        cookie: referencias.cookie(),
    };
    let mut envio = EnvioHttps;
    let mut rede = Contador::novo(&mut envio);
    let lido = ler_bloco_por_numero_via(&referencias, cli.unidade.as_deref(), bloco, &mut rede)?;
    let Leitura::Detalhe(detalhe) = lido.leitura else {
        return Err(crate::bloco_cli::FalhaLeitura::HtmlDeBlocosInvalido.into());
    };
    if !matches!(detalhe.tipo, TipoBloco::Interno | TipoBloco::Reuniao) {
        return Err(ErroPreviaProcessual::TipoInadequado.into());
    }
    let indice = IndiceLocal::da_configuracao(&arquivos);
    let aberto = obter_processo(
        &sessao,
        &leitura(&nup, cli.unidade.as_deref(), false),
        &indice,
        &mut rede,
    )?;
    if aberto.arvore.processo.nup != nup.as_str() {
        return Err(ErroPreviaProcessual::IdentidadeDoProcessoDivergente.into());
    }
    let presente = detalhe
        .itens
        .iter()
        .any(|item| item.processo == nup.as_str());
    if !presente && detalhe.itens.len() != detalhe.total_declarado {
        return Err(ErroPreviaProcessual::PertencimentoNaoComprovado.into());
    }
    if cli.json {
        let mut valor = json!({
            "bloco": bloco,
            "tipo_bloco": detalhe.tipo.rotulo(),
            "alvo": {"processo": nup.as_str()},
            "confirmado": false,
            "alterado": false,
            "presente_antes": presente,
            "presente_depois": presente,
            "origem": "web",
            "requisicoes": rede.feitas(),
        });
        indice_local::anexar_avisos(&mut valor, &indice);
        println!("{valor}");
        return Ok(());
    }
    indice_local::avisar_no_texto(&indice);
    if presente {
        println!(
            "Processo {} já está no bloco {} ({}); nenhuma inclusão necessária. Requisições: {}",
            nup.as_str(),
            bloco,
            detalhe.tipo.rotulo(),
            rede.feitas()
        );
    } else {
        println!(
            "Prévia: processo {} ausente do bloco {} ({}). Para incluir, repita com --confirmar quando a confirmação estiver disponível. Requisições: {}",
            nup.as_str(),
            bloco,
            detalhe.tipo.rotulo(),
            rede.feitas()
        );
    }
    Ok(())
}
