//! `ies processo concluir NUP [--confirmar]` (contracts/cli/processo-concluir.md):
//! NUP inválido sai com 2 antes de qualquer arquivo; prévia sem escrita;
//! com `--confirmar`, uma escrita conferida por leitura; efeito incerto, 3.
//! A saída nunca traz URL, `infra_hash` ou cookie.
use crate::indice_local::{self, IndiceLocal};
use crate::processo::NumeroProcesso;
use crate::processo_concluir::{
    DesfechoDaConclusao, PedidoDaConclusao, ProcessoConcluido, concluir_processo,
};
use crate::processo_obter::SessaoDoProcesso;
use crate::visao_do_controle::EnvioHttps;
use crate::{Cli, Domain, Processo, ProcessoAcao, configuracao, transporte};

/// `Some` só para `ies processo concluir`.
pub(crate) fn tentar_concluir(cli: &Cli) -> Option<anyhow::Result<()>> {
    let Domain::Processo(Processo {
        acao: Some(ProcessoAcao::Concluir { numero, confirmar }),
        ..
    }) = &cli.command
    else {
        return None;
    };
    let nup = numero.clone();
    Some(concluir(cli, &nup, *confirmar))
}

/// Lê os arquivos da sessão e chama `executar` com ela e o índice; a falha
/// de configuração ou de sessão sobe até o `main`, sem rede. Com `--json`,
/// `executar` põe os avisos do índice na saída (`anexar_avisos`); sem ele,
/// as linhas `Aviso do índice` saem aqui, em `stderr`, depois do sucesso.
pub(crate) fn com_sessao(
    cli: &Cli,
    executar: impl FnOnce(&SessaoDoProcesso<'_>, &IndiceLocal) -> anyhow::Result<()>,
) -> anyhow::Result<()> {
    let arquivos = configuracao::arquivos_de_acesso()?;
    let referencias = transporte::sessao::ReferenciasDaSessao::ler(
        &arquivos.url_controle,
        &arquivos.origem,
        &arquivos.cookie,
    )?;
    let sessao = SessaoDoProcesso {
        url_do_controle: referencias.url_do_controle(),
        origem: referencias.origem(),
        cookie: referencias.cookie(),
    };
    let indice = IndiceLocal::da_configuracao(&arquivos);
    executar(&sessao, &indice)?;
    // Na falha, `stderr` começa pelo erro (#695): os avisos só no sucesso.
    if !cli.json {
        indice_local::avisar_no_texto(&indice);
    }
    Ok(())
}

fn concluir(cli: &Cli, nup: &NumeroProcesso, confirmar: bool) -> anyhow::Result<()> {
    com_sessao(cli, |sessao, indice| {
        concluir_na_sessao(cli, sessao, indice, nup, confirmar)
    })
}

fn concluir_na_sessao(
    cli: &Cli,
    sessao: &SessaoDoProcesso<'_>,
    indice: &IndiceLocal,
    nup: &NumeroProcesso,
    confirmar: bool,
) -> anyhow::Result<()> {
    let pedido = PedidoDaConclusao {
        nup,
        unidade: cli.unidade.as_deref(),
        confirmar,
    };
    let feito = concluir_processo(sessao, &pedido, indice, &mut EnvioHttps)?;
    if cli.json {
        let mut valor = serde_json::to_value(resultado_conclusao(&feito))?;
        indice_local::anexar_avisos(&mut valor, indice);
        println!("{}", serde_json::to_string_pretty(&valor)?);
    } else {
        println!("{}", humano(&feito));
    }
    Ok(())
}

/// Projeção do sucesso: a função de domínio nunca constrói um JSON.
#[derive(serde::Serialize)]
struct ResultadoConclusao<'a> {
    aberto_em: &'a [String],
    alterado: bool,
    confirmado: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    depois: Option<&'a [String]>,
    depois_esperado: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    falha_do_envio: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    forma: Option<&'static str>,
    id_procedimento: &'a str,
    nup: &'a str,
    origem: &'static str,
    reconciliado_por_leitura: bool,
    requisicoes: u8,
    unidade: &'a str,
}

fn resultado_conclusao(feito: &ProcessoConcluido) -> ResultadoConclusao<'_> {
    ResultadoConclusao {
        nup: &feito.nup,
        id_procedimento: &feito.id_procedimento,
        unidade: &feito.unidade,
        origem: "web",
        confirmado: feito.confirmado(),
        alterado: feito.alterado(),
        forma: feito
            .forma()
            .map(crate::processo_concluir::FormaDaConclusao::como_texto),
        aberto_em: &feito.aberto_em,
        depois_esperado: feito.depois_esperado(),
        depois: feito.depois(),
        reconciliado_por_leitura: feito.reconciliado_por().is_some(),
        requisicoes: feito.requisicoes,
        falha_do_envio: feito.reconciliado_por().map(|f| format!("{f:?}")),
    }
}

/// Compatibilidade com os testes de contrato: o runtime serializa a struct
/// diretamente, sem construir `serde_json::Value` no caminho de sucesso.
#[cfg(test)]
pub(crate) fn json_de_sucesso(feito: &ProcessoConcluido) -> serde_json::Value {
    serde_json::to_value(resultado_conclusao(feito))
        .expect("a projeção tipada de sucesso só possui valores JSON")
}

pub(crate) fn lista_humana(siglas: &[&str]) -> String {
    if siglas.is_empty() {
        "(nenhuma unidade)".to_owned()
    } else {
        siglas.join(", ")
    }
}

/// Saída humana do sucesso.
pub(crate) fn humano(feito: &ProcessoConcluido) -> String {
    let refs = |v: &[String]| lista_humana(&v.iter().map(String::as_str).collect::<Vec<_>>());
    let mut linhas = vec![
        format!("Processo {} na unidade {}", feito.nup, feito.unidade),
        format!("Aberto em: {}", refs(&feito.aberto_em)),
    ];
    match &feito.desfecho {
        DesfechoDaConclusao::NaoAbertoNaUnidade { .. } => linhas.push(
            "Nada a fazer: o processo não está aberto nesta unidade e a barra não oferece concluir."
                .to_owned(),
        ),
        DesfechoDaConclusao::Previa { forma } => {
            linhas.push(format!("Ação na barra: {}", forma.como_texto()));
            linhas.push(format!("Depois de concluir: {}", refs(&feito.depois_esperado())));
            linhas.push("Para concluir, repita com --confirmar".to_owned());
        }
        DesfechoDaConclusao::Concluido {
            depois,
            reconciliado_por,
            ..
        } => {
            linhas.push(format!("Concluído. Aberto em, relido: {}", refs(depois)));
            if let Some(falha) = reconciliado_por {
                linhas.push(format!(
                    "A resposta não foi a esperada ({falha:?}); a leitura confirmou."
                ));
            }
        }
    }
    linhas.push(format!("Requisições: {}", feito.requisicoes));
    linhas.join("\n")
}

#[cfg(test)]
#[path = "processo_concluir_cli/tests.rs"]
mod tests;
