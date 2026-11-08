//! `ies unidade trocar SIGLA [--confirmar]` (#205;
//! contracts/cli/unidade-trocar.md): uso inválido sai com 2 antes da
//! configuração; com `--confirmar`, o lock do login antes de qualquer
//! pedido; troca verificada regrava a URL da sessão e o marcador, apaga os
//! links assinados da instalação e colhe o Controle novo. A troca vale para
//! toda a sessão da conta, e a saída diz isso.
use crate::configuracao::{self, ArquivosDeAcesso};
use crate::formulario_web::{RespostaDoPost, SubmissaoDerivada};
use crate::indice_local::IndiceLocal;
use crate::instalacao::Instalacao;
use crate::saida_erro;
use crate::transporte::FalhaTransporte;
use crate::transporte::contagem::ContagemDeRequisicoes;
use crate::transporte::sessao::{CookieDaSessao, ReferenciasDaSessao};
use crate::unidade_trocar::{
    ACAO_DE_TROCAR, ControleLido, EnvioDaTroca, EnvioHttpsDaTroca, ErroTroca, PedidoDeTroca,
    SessaoDaTroca, Troca, UnidadeOferecida, trocar_unidade,
};
use serde_json::{Map, Value, json};
use std::process::ExitCode;

const AVISO_DE_ESCOPO: &str =
    "a unidade vale para toda a sessão da conta: todo agente que usa esta sessão passa a vê-la";

/// Conta cada pedido feito, sem alterar nenhum.
struct Contador<V> {
    envio: V,
    pedidos: ContagemDeRequisicoes,
}

impl<V: EnvioDaTroca> EnvioDaTroca for Contador<V> {
    type Corpo = V::Corpo;
    fn get(
        &mut self,
        url: &str,
        cookie: &str,
    ) -> Result<RespostaDoPost<Self::Corpo>, FalhaTransporte> {
        self.pedidos.registrar();
        self.envio.get(url, cookie)
    }
    fn post(
        &mut self,
        submissao: SubmissaoDerivada,
        cookie: &str,
    ) -> Result<RespostaDoPost<Self::Corpo>, FalhaTransporte> {
        self.pedidos.registrar();
        self.envio.post(submissao, cookie)
    }
}

pub(crate) fn trocar(
    sigla: &str,
    confirmar: bool,
    unidade: Option<&str>,
    json_mode: bool,
) -> ExitCode {
    let sigla = sigla.trim();
    if sigla.is_empty() {
        return uso_invalido(json_mode);
    }
    let arquivos = match configuracao::arquivos_de_acesso() {
        Ok(arquivos) => arquivos,
        Err(erro) => return crate::erro_fonte(json_mode, erro.subcodigo(), erro.mensagem()),
    };
    // Nenhum `ies login` troca a sessão no meio da troca: com `--confirmar`,
    // o retrato da sessão é lido já sob o lock (#222).
    let _trava = if confirmar {
        match crate::sessao_login::TravaDoLogin::obter(
            &arquivos.url_controle,
            crate::sessao_login_cli::ESPERA_DO_LOCK,
        ) {
            Ok(trava) => Some(trava),
            Err(erro) => return crate::erro_fonte(json_mode, erro.subcodigo(), erro.mensagem()),
        }
    } else {
        None
    };
    let referencias =
        match ReferenciasDaSessao::ler(&arquivos.url_controle, &arquivos.origem, &arquivos.cookie)
            .and_then(|r| r.conferir_controle().map(|()| r))
        {
            Ok(r) => r,
            Err(f) => {
                return saida_erro::erro_fonte_http(
                    json_mode,
                    f.subcodigo(),
                    f.mensagem(),
                    f.status_http(),
                );
            }
        };
    let Some(instalacao) = Instalacao::interpretar(referencias.origem()) else {
        let f = FalhaTransporte::OrigemNaoAutorizada;
        return crate::erro_fonte(json_mode, f.subcodigo(), f.mensagem());
    };
    let sessao = SessaoDaTroca {
        url_do_controle: referencias.url_do_controle(),
        instalacao: &instalacao,
        cookie: referencias.cookie(),
    };
    let pedido = PedidoDeTroca {
        sigla,
        confirmar,
        unidade_esperada: unidade,
    };
    let mut envio = Contador {
        envio: EnvioHttpsDaTroca,
        pedidos: ContagemDeRequisicoes::default(),
    };
    let resultado = trocar_unidade(&sessao, &pedido, &mut envio);
    let indice = IndiceLocal::da_configuracao(&arquivos);
    // A saída histórica é u32; a contagem real satura em vez de transbordar.
    let pedidos = u32::try_from(envio.pedidos.total()).unwrap_or(u32::MAX);
    match resultado {
        Ok(troca) => concluir(
            json_mode,
            &arquivos,
            &indice,
            (&instalacao, &referencias),
            &troca,
            pedidos,
        ),
        Err(erro) => {
            if matches!(erro, ErroTroca::EfeitoIncerto { .. }) {
                // A unidade pode ter mudado: nada da anterior é reaproveitado.
                indice.esquecer_links_da_instalacao(&instalacao);
            }
            publicar_falha(json_mode, &erro, pedidos)
        }
    }
}

fn uso_invalido(json_mode: bool) -> ExitCode {
    let mensagem = "a sigla da unidade está vazia";
    if json_mode {
        println!(
            "{}",
            json!({"codigo": "entrada_invalida", "subcodigo": "sigla_de_unidade_vazia",
                   "erro": mensagem, "retentavel": false})
        );
    } else {
        eprintln!("IES: {mensagem} [sigla_de_unidade_vazia]");
    }
    ExitCode::from(2)
}

/// Causas diferentes permanecem tipadas até a apresentação na CLI.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum FalhaAplicarTroca {
    Cookie(FalhaTransporte),
    Gravacao(crate::sessao_login::ErroLogin),
}

/// Depois da troca verificada: grava a URL do Controle novo (e o
/// marcador) como a da sessão, apaga os links assinados da instalação e
/// colhe o Controle novo com a chave da unidade nova. O cookie é o do
/// retrato que fez a troca (lido sob o lock). Devolve quantos links o
/// índice apagou (`None` com o índice desligado).
pub(crate) fn aplicar_troca(
    arquivos: &ArquivosDeAcesso,
    indice: &IndiceLocal,
    instalacao: &Instalacao,
    cookie: &CookieDaSessao,
    novo: &ControleLido,
) -> Result<Option<usize>, FalhaAplicarTroca> {
    let cookie = cookie.valor().map_err(FalhaAplicarTroca::Cookie)?;
    crate::sessao_login::regravar_url_do_controle(arquivos, &novo.url, cookie)
        .map_err(FalhaAplicarTroca::Gravacao)?;
    let descartados = indice.esquecer_links_da_instalacao(instalacao);
    crate::colheita_do_controle::Colheita::do_cookie(indice, instalacao, cookie, &novo.url)
        .colher(&novo.html, Some(&novo.mime));
    Ok(descartados)
}

fn concluir(
    json_mode: bool,
    arquivos: &ArquivosDeAcesso,
    indice: &IndiceLocal,
    (instalacao, sessao): (&Instalacao, &ReferenciasDaSessao),
    troca: &Troca,
    pedidos: u32,
) -> ExitCode {
    let mut descartados = None;
    if let Some(novo) = &troca.controle_novo {
        match aplicar_troca(arquivos, indice, instalacao, sessao.cookie(), novo) {
            Ok(n) => descartados = n,
            Err(_causa) => {
                // A saída mantém o subcódigo legado; a origem não é mais perdida na função.
                let subcodigo = "url_do_controle_da_unidade_nova_nao_gravada";
                let mensagem = "a unidade foi trocada e conferida, mas a URL da sessão não foi gravada; rode ies login --forcar";
                if json_mode {
                    println!(
                        "{}",
                        json!({"codigo": "fonte_invalida", "subcodigo": subcodigo, "erro": mensagem,
                               "retentavel": false, "depois": troca.pedida, "requisicoes": pedidos})
                    );
                } else {
                    eprintln!("IES: {mensagem} [{subcodigo}]");
                }
                return ExitCode::from(1);
            }
        }
    } else {
        let inicial = &troca.controle_inicial;
        crate::colheita_do_controle::Colheita::das_referencias(indice, sessao)
            .colher(&inicial.html, Some(&inicial.mime));
    }
    if json_mode {
        if let Err(erro) =
            crate::saida::imprimir_json(&resultado_troca(troca, descartados, pedidos))
        {
            eprintln!("IES: falha ao emitir JSON: {erro}");
            return std::process::ExitCode::from(1);
        }
    } else {
        println!("{}", humano(troca, pedidos));
    }
    ExitCode::SUCCESS
}

fn json_da_unidade(u: &UnidadeOferecida) -> Value {
    json!({"sigla": u.sigla, "descricao": u.descricao, "id_unidade": u.id_unidade})
}

fn json_das_unidades(lista: &[UnidadeOferecida]) -> Value {
    Value::Array(lista.iter().map(json_da_unidade).collect())
}

/// A representação externa expõe somente os campos observados da unidade.
#[derive(serde::Serialize)]
struct UnidadePublica<'a> {
    descricao: &'a str,
    id_unidade: &'a str,
    sigla: &'a str,
}

#[derive(serde::Serialize)]
struct EnvioTrocaPublico<'a> {
    acao: &'static str,
    campos: &'a [String],
    metodo: &'static str,
}

#[derive(serde::Serialize)]
struct ResultadoTroca<'a> {
    alterado: bool,
    antes: &'a str,
    confirmado: bool,
    depois: Option<&'a str>,
    envio: Option<EnvioTrocaPublico<'a>>,
    escopo: &'static str,
    id_unidade: Option<&'a str>,
    links_descartados: Option<usize>,
    oferecidas: Option<Vec<UnidadePublica<'a>>>,
    pedida: &'a str,
    requisicoes: u32,
}

fn resultado_troca(troca: &Troca, descartados: Option<usize>, pedidos: u32) -> ResultadoTroca<'_> {
    let alterado = troca.controle_novo.is_some();
    let ja_na_unidade = troca.escolhida.is_none();
    ResultadoTroca {
        alterado,
        antes: &troca.antes,
        confirmado: troca.confirmado,
        depois: (alterado || ja_na_unidade).then_some(troca.pedida.as_str()),
        envio: troca
            .campos_do_envio
            .as_deref()
            .map(|campos| EnvioTrocaPublico {
                acao: ACAO_DE_TROCAR,
                campos,
                metodo: "POST",
            }),
        escopo: "sessao_da_conta",
        id_unidade: troca.escolhida.as_ref().map(|u| u.id_unidade.as_str()),
        links_descartados: descartados,
        oferecidas: (!ja_na_unidade).then(|| {
            troca
                .oferecidas
                .iter()
                .map(|u| UnidadePublica {
                    descricao: &u.descricao,
                    id_unidade: &u.id_unidade,
                    sigla: &u.sigla,
                })
                .collect()
        }),
        pedida: &troca.pedida,
        requisicoes: pedidos,
    }
}

/// Contrato legado usado somente como oráculo dos testes de paridade.
#[cfg(test)]
pub(crate) fn json_de_sucesso(troca: &Troca, descartados: Option<usize>, pedidos: u32) -> Value {
    let alterado = troca.controle_novo.is_some();
    let envio = troca.campos_do_envio.as_ref().map_or(
        Value::Null,
        |campos| json!({"metodo": "POST", "acao": ACAO_DE_TROCAR, "campos": campos}),
    );
    let ja_na_unidade = troca.escolhida.is_none();
    json!({
        "antes": troca.antes,
        "pedida": troca.pedida,
        "id_unidade": troca.escolhida.as_ref().map(|u| u.id_unidade.clone()),
        "depois": if alterado || ja_na_unidade { json!(troca.pedida) } else { Value::Null },
        "confirmado": troca.confirmado,
        "alterado": alterado,
        "escopo": "sessao_da_conta",
        "oferecidas": if ja_na_unidade { Value::Null } else { json_das_unidades(&troca.oferecidas) },
        "envio": envio,
        "links_descartados": descartados,
        "requisicoes": pedidos,
    })
}

fn humano(troca: &Troca, pedidos: u32) -> String {
    let mut linhas = Vec::new();
    match (&troca.escolhida, &troca.controle_novo) {
        (None, _) => linhas.push(format!(
            "IES: a sessão já está na unidade {}; nada enviado.",
            troca.pedida
        )),
        (Some(_), Some(_)) => linhas.push(format!(
            "IES: unidade trocada de {} para {} e conferida no Controle.",
            troca.antes, troca.pedida
        )),
        (Some(u), None) => {
            linhas.push(format!(
                "IES: prévia, nada enviado: a sessão está em {} e a tela oferece {} ({}).",
                troca.antes, u.sigla, u.descricao
            ));
            let siglas: Vec<_> = troca.oferecidas.iter().map(|u| u.sigla.as_str()).collect();
            linhas.push(format!("Unidades oferecidas: {}", siglas.join(", ")));
            if let Some(campos) = &troca.campos_do_envio {
                linhas.push(format!("POST que seria enviado: {}", campos.join(", ")));
            }
            linhas.push("Para trocar, repita com --confirmar.".to_owned());
        }
    }
    linhas.push(format!("IES: {AVISO_DE_ESCOPO}; {pedidos} requisições."));
    linhas.join("\n")
}

fn json_de_falha(erro: &ErroTroca, pedidos: u32) -> Value {
    let mut valor =
        saida_erro::json_de_erro_fonte(erro.subcodigo(), erro.mensagem(), erro.status_http());
    let Some(objeto) = valor.as_object_mut() else {
        return valor;
    };
    objeto.insert("codigo".to_owned(), json!(erro.codigo()));
    objeto.insert("requisicoes".to_owned(), json!(pedidos));
    inserir_detalhes(objeto, erro);
    valor
}

fn inserir_detalhes(objeto: &mut Map<String, Value>, erro: &ErroTroca) {
    match erro {
        ErroTroca::UnidadeNaoOferecida { oferecidas } => {
            objeto.insert("oferecidas".to_owned(), json_das_unidades(oferecidas));
        }
        ErroTroca::SiglaDeUnidadeAmbigua { candidatas } => {
            objeto.insert("candidatas".to_owned(), json_das_unidades(candidatas));
        }
        ErroTroca::EfeitoIncerto { etapa, falha } => {
            objeto.insert("etapa".to_owned(), json!(etapa.nome()));
            objeto.insert("falha".to_owned(), json!(falha.subcodigo()));
            objeto.insert("escopo".to_owned(), json!("sessao_da_conta"));
        }
        _ => {}
    }
}

/// A saída humana parte exclusivamente da falha tipada, sem montar JSON.
fn linhas_da_falha(erro: &ErroTroca, pedidos: u32) -> Vec<String> {
    let mut linhas = vec![saida_erro::linha_humana(
        erro.subcodigo(),
        erro.mensagem(),
        erro.status_http(),
    )];
    match erro {
        ErroTroca::UnidadeNaoOferecida { oferecidas } => {
            let siglas: Vec<_> = oferecidas.iter().map(|u| u.sigla.as_str()).collect();
            linhas.push(format!("IES: oferecidas: {}", siglas.join(", ")));
        }
        ErroTroca::SiglaDeUnidadeAmbigua { candidatas } => {
            let siglas: Vec<_> = candidatas.iter().map(|u| u.sigla.as_str()).collect();
            linhas.push(format!("IES: candidatas: {}", siglas.join(", ")));
        }
        ErroTroca::EfeitoIncerto { etapa, falha } => {
            linhas.push(format!(
                "IES: etapa {}: {}; {AVISO_DE_ESCOPO}",
                etapa.nome(),
                falha.subcodigo()
            ));
        }
        _ => {}
    }
    linhas.push(format!("IES: {pedidos} requisições."));
    linhas
}

fn publicar_falha(json_mode: bool, erro: &ErroTroca, pedidos: u32) -> ExitCode {
    if json_mode {
        println!("{}", json_de_falha(erro, pedidos));
    } else {
        for linha in linhas_da_falha(erro, pedidos) {
            eprintln!("{linha}");
        }
    }
    ExitCode::from(erro.saida())
}

#[cfg(test)]
#[path = "unidade_trocar_cli/tests.rs"]
mod tests;

#[cfg(test)]
#[path = "unidade_trocar_cli/tests_contagem.rs"]
mod tests_contagem;
