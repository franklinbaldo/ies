//! `ies unidade trocar SIGLA [--confirmar]` (#205;
//! contracts/cli/unidade-trocar.md): uso inválido sai com 2 antes da
//! configuração; com `--confirmar`, o lock do login antes de qualquer
//! pedido; troca verificada regrava a URL da sessão e o marcador, apaga os
//! links assinados da instalação e colhe o Controle novo. A troca vale para
//! toda a sessão da conta, e a saída diz isso.
use crate::acao_do_sei::AcaoDoSei;
use crate::configuracao::{self, ArquivosDeAcesso};
use crate::formulario_web::{RespostaDoPost, SubmissaoDerivada};
use crate::indice_local::IndiceLocal;
use crate::instalacao::Instalacao;
use crate::transporte::FalhaTransporte;
use crate::transporte::contagem::ContagemDeRequisicoes;
use crate::transporte::sessao::{CookieDaSessao, ReferenciasDaSessao};
use crate::unidade_trocar::{
    ControleLido, EnvioDaTroca, EnvioHttpsDaTroca, ErroTroca, PedidoDeTroca, SessaoDaTroca, Troca,
    trocar_unidade,
};

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

/// A sigla da tela de troca (parser do clap: vazia é saída 2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) struct SiglaDaUnidadeVazia;

pub(crate) fn sigla_da_troca(sigla: &str) -> Result<String, SiglaDaUnidadeVazia> {
    let sigla = sigla.trim();
    if sigla.is_empty() {
        return Err(SiglaDaUnidadeVazia);
    }
    Ok(sigla.to_owned())
}

/// A troca falhou; os pedidos feitos (inclusive o POST, se houve) vão junto.
#[derive(Debug, thiserror::Error)]
#[error("{self:?}")]
pub(crate) struct TrocaNaoFeita {
    #[source]
    pub(crate) erro: ErroTroca,
    pub(crate) requisicoes: u32,
}

/// A troca foi enviada e conferida, mas a sessão local não foi regravada:
/// a conta já está em `depois`; rodar `ies login --forcar`.
#[derive(Debug, thiserror::Error)]
#[error("{self:?}")]
pub(crate) struct TrocaFeitaSemRegravarSessao {
    #[source]
    pub(crate) causa: FalhaAplicarTroca,
    pub(crate) depois: String,
    pub(crate) requisicoes: u32,
}

/// `sigla`: já conferida pelo clap ([`sigla_da_troca`]).
pub(crate) fn trocar(
    sigla: &str,
    confirmar: bool,
    unidade: Option<&str>,
    json_mode: bool,
) -> anyhow::Result<()> {
    let arquivos = configuracao::arquivos_de_acesso()?;
    // Nenhum `ies login` troca a sessão no meio da troca: com `--confirmar`,
    // o retrato da sessão é lido já sob o lock (#222).
    let _trava = if confirmar {
        Some(crate::sessao_login::TravaDoLogin::obter(
            &arquivos.url_controle,
            crate::sessao_login_cli::ESPERA_DO_LOCK,
        )?)
    } else {
        None
    };
    let referencias =
        ReferenciasDaSessao::ler(&arquivos.url_controle, &arquivos.origem, &arquivos.cookie)?;
    referencias.conferir_controle()?;
    let instalacao = Instalacao::interpretar(referencias.origem())
        .ok_or(FalhaTransporte::OrigemNaoAutorizada)?;
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
    let troca = resultado.map_err(|erro| {
        if matches!(erro, ErroTroca::NaoConfirmado { .. }) {
            // A unidade pode ter mudado: nada da anterior é reaproveitado.
            indice.esquecer_links_da_instalacao(&instalacao);
        }
        TrocaNaoFeita {
            erro,
            requisicoes: pedidos,
        }
    })?;
    concluir(
        json_mode,
        &arquivos,
        &indice,
        (&instalacao, &referencias),
        &troca,
        pedidos,
    )
}

/// Causas diferentes permanecem tipadas.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum FalhaAplicarTroca {
    Cookie(#[source] FalhaTransporte),
    Gravacao(#[source] crate::sessao_login::ErroLogin),
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
) -> anyhow::Result<()> {
    let mut descartados = None;
    if let Some(novo) = &troca.controle_novo {
        descartados = aplicar_troca(arquivos, indice, instalacao, sessao.cookie(), novo).map_err(
            |causa| TrocaFeitaSemRegravarSessao {
                causa,
                depois: troca.pedida.clone(),
                requisicoes: pedidos,
            },
        )?;
    } else {
        let inicial = &troca.controle_inicial;
        crate::colheita_do_controle::Colheita::das_referencias(indice, sessao)
            .colher(&inicial.html, Some(&inicial.mime));
    }
    if json_mode {
        println!(
            "{}",
            serde_json::to_string_pretty(&resultado_troca(troca, descartados, pedidos))
                .expect("JSON serializável")
        );
    } else {
        println!("{}", humano(troca, pedidos));
    }
    Ok(())
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
    acao: AcaoDoSei,
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
                acao: AcaoDoSei::InfraTrocarUnidade,
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

#[cfg(test)]
use crate::unidade_trocar::UnidadeOferecida;
#[cfg(test)]
use serde_json::{Value, json};

#[cfg(test)]
fn json_da_unidade(u: &UnidadeOferecida) -> Value {
    json!({"sigla": u.sigla, "descricao": u.descricao, "id_unidade": u.id_unidade})
}

#[cfg(test)]
fn json_das_unidades(lista: &[UnidadeOferecida]) -> Value {
    Value::Array(lista.iter().map(json_da_unidade).collect())
}

/// Contrato legado usado somente como oráculo dos testes de paridade.
#[cfg(test)]
pub(crate) fn json_de_sucesso(troca: &Troca, descartados: Option<usize>, pedidos: u32) -> Value {
    let alterado = troca.controle_novo.is_some();
    let envio = troca.campos_do_envio.as_ref().map_or(
        Value::Null,
        |campos| json!({"metodo": "POST", "acao": AcaoDoSei::InfraTrocarUnidade, "campos": campos}),
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

#[cfg(test)]
#[path = "unidade_trocar_cli/tests.rs"]
mod tests;

#[cfg(test)]
#[path = "unidade_trocar_cli/tests_contagem.rs"]
mod tests_contagem;
