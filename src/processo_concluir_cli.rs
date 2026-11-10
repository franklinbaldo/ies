//! `ies processo concluir NUP [--confirmar]` (contracts/cli/processo-concluir.md):
//! NUP inválido sai com 2 antes de qualquer arquivo; prévia sem escrita;
//! com `--confirmar`, uma escrita conferida por leitura; efeito incerto, 3.
//! A saída nunca traz URL, `infra_hash` ou cookie.
use crate::indice_local::IndiceLocal;
use crate::processo::NumeroProcesso;
use crate::processo_concluir::{
    ErroConcluir, FalhaDoEnvio, PedidoDaConclusao, ProcessoConcluido, concluir_processo,
};
use crate::processo_obter::SessaoDoProcesso;
use crate::transporte::StatusHttpRecusado;
use crate::visao_do_controle::EnvioHttps;
use crate::{
    Cli, Domain, Processo, ProcessoAcao, configuracao, erro_fonte, saida_erro, transporte,
};
#[path = "processo_concluir_cli/falha.rs"]
pub(crate) mod falha;
use falha::FalhaDaSituacao;
use serde_json::{Value, json};
use std::process::ExitCode;

/// `Some` só para `ies processo concluir`.
pub(crate) fn tentar_concluir(cli: &Cli) -> Option<ExitCode> {
    let Domain::Processo(Processo {
        acao: Some(ProcessoAcao::Concluir { numero, confirmar }),
        ..
    }) = &cli.command
    else {
        return None;
    };
    Some(match NumeroProcesso::novo(numero) {
        Ok(nup) => concluir(cli, &nup, *confirmar),
        Err(_) => erro_de_uso(cli.json),
    })
}

pub(crate) fn erro_de_uso(json_mode: bool) -> ExitCode {
    if json_mode {
        println!(
            "{}",
            json!({
                "codigo": "entrada_invalida",
                "erro": "número de processo inválido",
                "retentavel": false
            })
        );
    } else {
        eprintln!("IES: número de processo inválido");
    }
    ExitCode::from(2)
}

/// Lê os arquivos da sessão e chama `executar` com ela e o índice; a falha
/// de configuração sai com 1 `fonte_invalida`, sem rede (também a de
/// `processo reabrir`).
pub(crate) fn com_sessao(
    cli: &Cli,
    executar: impl FnOnce(&SessaoDoProcesso<'_>, &IndiceLocal) -> ExitCode,
) -> ExitCode {
    let arquivos = match configuracao::arquivos_de_acesso() {
        Ok(arquivos) => arquivos,
        Err(erro) => return erro_fonte(cli.json, erro.mensagem()),
    };
    let lidos = transporte::sessao::ReferenciasDaSessao::ler(
        &arquivos.url_controle,
        &arquivos.origem,
        &arquivos.cookie,
    );
    let referencias = match lidos {
        Ok(referencias) => referencias,
        Err(falha) => return erro_fonte(cli.json, falha.mensagem()),
    };
    let sessao = SessaoDoProcesso {
        url_do_controle: referencias.url_do_controle(),
        origem: referencias.origem(),
        cookie: referencias.cookie(),
    };
    executar(&sessao, &IndiceLocal::da_configuracao(&arquivos))
}

fn concluir(cli: &Cli, nup: &NumeroProcesso, confirmar: bool) -> ExitCode {
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
) -> ExitCode {
    let pedido = PedidoDaConclusao {
        nup,
        unidade: cli.unidade.as_deref(),
        confirmar,
    };
    match concluir_processo(sessao, &pedido, indice, &mut EnvioHttps) {
        Ok(feito) => {
            if cli.json {
                if let Err(erro) = crate::saida::imprimir_json(&resultado_conclusao(&feito)) {
                    eprintln!("IES: falha ao emitir JSON: {erro}");
                    return std::process::ExitCode::from(1);
                }
            } else {
                println!("{}", humano(&feito));
            }
            ExitCode::SUCCESS
        }
        Err(ErroConcluir::Abertura(erro)) => {
            saida_erro::erro_fonte_http(cli.json, erro.mensagem(), erro.status_http())
        }
        Err(erro) => publicar_falha(cli.json, &erro),
    }
}

/// O tipo original de `FalhaDoEnvio` permanece intacto: só projetamos
/// seus dados observáveis na fronteira JSON.
pub(crate) fn falha_do_envio_publica(falha: &FalhaDoEnvio) -> crate::saida_erro::FalhaPublica {
    crate::saida_erro::FalhaPublica::nova(falha.status_http())
}

/// Projeção do sucesso: a função de domínio nunca constrói um JSON.
#[derive(serde::Serialize)]
struct ResultadoConclusao<'a> {
    aberto_em: &'a [String],
    alterado: bool,
    confirmado: bool,
    depois: Option<&'a [String]>,
    depois_esperado: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    falha_do_envio: Option<crate::saida_erro::FalhaPublica>,
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
        confirmado: feito.confirmado,
        alterado: feito.alterado,
        forma: feito
            .forma
            .map(crate::processo_concluir::FormaDaConclusao::como_texto),
        aberto_em: &feito.aberto_em,
        depois_esperado: feito.depois_esperado(),
        depois: feito.depois.as_deref(),
        reconciliado_por_leitura: feito.reconciliado_por.is_some(),
        requisicoes: feito.requisicoes,
        falha_do_envio: feito.reconciliado_por.as_ref().map(falha_do_envio_publica),
    }
}

/// Compatibilidade com os testes de contrato: o runtime serializa a struct
/// diretamente, sem construir `serde_json::Value` no caminho de sucesso.
#[cfg(test)]
pub(crate) fn json_de_sucesso(feito: &ProcessoConcluido) -> Value {
    serde_json::to_value(resultado_conclusao(feito))
        .expect("a projeção tipada de sucesso só possui valores JSON")
}

/// O corpo comum das falhas depois da abertura (também as de `processo
/// reabrir`): códigos, mensagem e o status HTTP, quando houver.
pub(crate) fn json_base_da_falha(
    codigo: &str,
    mensagem: &str,
    status: Option<StatusHttpRecusado>,
) -> Value {
    serde_json::to_value(FalhaDaSituacao::nova(codigo, mensagem, status))
        .expect("a projeção de falha só possui valores JSON")
}

/// JSON da falha depois da abertura.
pub(crate) fn json_de_falha(erro: &ErroConcluir) -> FalhaDaSituacao<'_> {
    let mut valor = FalhaDaSituacao::nova(erro.codigo(), erro.mensagem(), erro.status_http());
    match erro {
        ErroConcluir::SituacaoDoProcessoNaoLidaAntesDeConcluir { motivo } => {
            valor.motivo = Some(motivo.como_texto());
        }
        ErroConcluir::UnidadeDaSessaoAusenteDaSituacao { aberto_em }
        | ErroConcluir::ConclusaoNaoPublicadaNaBarra { aberto_em } => {
            valor.aberto_em = Some(aberto_em);
        }
        ErroConcluir::ConclusaoNaoAplicada {
            aberto_em,
            depois,
            falha_do_envio,
        } => {
            valor.aberto_em = Some(aberto_em);
            valor.depois = Some(depois);
            valor.falha_do_envio = falha_do_envio.as_ref().map(falha_do_envio_publica);
        }
        ErroConcluir::EfeitoIncertoNaConclusao {
            aberto_em,
            falha_do_envio,
            verificacao,
        } => {
            valor.aberto_em = Some(aberto_em);
            valor.verificacao = Some(verificacao.motivo());
            valor.falha_do_envio = falha_do_envio.as_ref().map(falha_do_envio_publica);
        }
        _ => {}
    }
    valor
}

fn publicar_falha(json_mode: bool, erro: &ErroConcluir) -> ExitCode {
    if json_mode {
        if let Err(falha) = crate::saida::imprimir_json_compacto(&json_de_falha(erro)) {
            eprintln!("IES: falha ao emitir JSON: {falha}");
        }
    } else {
        for linha in linhas_da_falha(erro) {
            eprintln!("{linha}");
        }
    }
    ExitCode::from(erro.saida())
}

/// Publica a falha já em JSON (também a de `processo reabrir`): o JSON, ou
/// a mensagem, as listas, a resposta da escrita e a verificação no stderr.
pub(crate) fn publicar_valor_da_falha(json_mode: bool, valor: &Value, saida: u8) -> ExitCode {
    if json_mode {
        println!("{valor}");
    } else {
        eprintln!("IES: {}", valor["erro"].as_str().unwrap_or_default());
        for campo in ["aberto_em", "depois"] {
            if let Some(lista) = valor[campo].as_array() {
                let siglas: Vec<_> = lista.iter().filter_map(Value::as_str).collect();
                eprintln!("IES: {campo}: {}", lista_humana(&siglas));
            }
        }
        if let Some(motivo) = valor["verificacao"].as_str() {
            eprintln!("IES: verificação: {motivo}");
        }
    }
    ExitCode::from(saida)
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
    match (feito.forma, feito.confirmado, &feito.depois) {
        (None, _, _) => linhas.push(
            "Nada a fazer: o processo não está aberto nesta unidade e a barra não oferece concluir."
                .to_owned(),
        ),
        (Some(forma), false, _) => {
            linhas.push(format!("Ação na barra: {}", forma.como_texto()));
            linhas.push(format!("Depois de concluir: {}", refs(&feito.depois_esperado())));
            linhas.push("Para concluir, repita com --confirmar".to_owned());
        }
        (Some(_), true, depois) => {
            let depois = depois.as_deref().unwrap_or_default();
            linhas.push(format!("Concluído. Aberto em, relido: {}", refs(depois)));
            if let Some(falha) = &feito.reconciliado_por {
                linhas.push(format!(
                    "A resposta não foi a esperada ({falha:?}); a leitura confirmou."
                ));
            }
        }
    }
    linhas.push(format!("Requisições: {}", feito.requisicoes));
    linhas.join("\n")
}

/// Mensagens humanas projetadas diretamente do erro de conclusão, sem
/// montar nem interpretar a representação JSON destinada aos agentes.
fn linhas_da_falha(erro: &ErroConcluir) -> Vec<String> {
    use ErroConcluir as E;

    let mut linhas = vec![format!("IES: {}", erro.mensagem())];
    let aberto_em = match erro {
        E::UnidadeDaSessaoAusenteDaSituacao { aberto_em }
        | E::ConclusaoNaoPublicadaNaBarra { aberto_em }
        | E::ConclusaoNaoAplicada { aberto_em, .. }
        | E::EfeitoIncertoNaConclusao { aberto_em, .. } => Some(aberto_em.as_slice()),
        _ => None,
    };
    let depois = match erro {
        E::ConclusaoNaoAplicada { depois, .. } => Some(depois.as_slice()),
        _ => None,
    };
    for (nome, unidades) in [("aberto_em", aberto_em), ("depois", depois)] {
        if let Some(unidades) = unidades {
            let siglas: Vec<_> = unidades.iter().map(String::as_str).collect();
            linhas.push(format!("IES: {nome}: {}", lista_humana(&siglas)));
        }
    }
    let falha_do_envio = match erro {
        E::ConclusaoNaoAplicada { falha_do_envio, .. }
        | E::EfeitoIncertoNaConclusao { falha_do_envio, .. } => falha_do_envio.as_ref(),
        _ => None,
    };
    if let Some(falha) = falha_do_envio {
        linhas.push(format!("IES: resposta da escrita: {falha:?}"));
    }
    if let E::EfeitoIncertoNaConclusao { verificacao, .. } = erro
        && let Some(motivo) = verificacao.motivo()
    {
        linhas.push(format!("IES: verificação: {motivo}"));
    }
    linhas
}

#[cfg(test)]
#[path = "processo_concluir_cli/tests.rs"]
mod tests;
