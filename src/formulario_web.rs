//! Submissão derivada de formulário HTML servido pelo SEI, sem rede.
//! Contrato: docs/arquitetura/0002-submeter-formulario-observado.md.
//! `action` aqui é o atributo do `<form>`, não um despachante genérico.
mod ajax;
mod codificacao;
mod erro;
mod guardada;
mod leitura;
mod login;
mod redirecionamento;
mod unidade;

use crate::acao_do_sei::{AcaoDaSubmissao, AcaoDoSei};
use crate::html_atributos::{nos_com_id, nos_por_elemento_e_classe};
use crate::instalacao::Instalacao;
use crate::transporte::sessao::CookieDaSessao;
use crate::transporte::validacao_da_infra::mensagem_de_validacao_da_infra;
use crate::transporte::{FalhaTransporte, RespostaWeb, decodificar_resposta};
pub(crate) use ajax::{ConsultaAjax, derivar_consulta_ajax};
use codificacao::codificar;
pub(crate) use codificacao::decodificar_atributo;
pub(crate) use erro::ErroSubmissaoFormulario;
pub(crate) use guardada::derivar_submissao_guardada;
pub(crate) use leitura::localizar_formulario;
use leitura::{
    Controle, UrlHttps, conferir_metodo_e_codificacao, ler_controles, resolver_action,
    resolver_action_declarada,
};
pub(crate) use login::derivar_submissao_de_login;
pub(crate) use redirecionamento::consulta_do_controlador;
use std::fmt;
use std::io::Read;
use unidade::conferir_unidade;

mod estado;
pub(crate) use estado::{EstadoDeSessao, EstadoDoProcesso, PreferenciaDoUsuario};

/// Avaliação obrigatória por operação, sem valor padrão.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum EfeitoAvaliado {
    ConsultaSemEfeitoObservado {
        evidencia: &'static str,
    },
    AlteraEstadoDaSessao {
        o_que: EstadoDeSessao,
    },
    /// Escrita: grava uma preferência que vale além da sessão; exige prévia,
    /// confirmação e verificação por leitura.
    AlteraPreferenciaDoUsuario {
        o_que: PreferenciaDoUsuario,
    },
    /// Escrita: muda o estado do processo para todos os usuários; exige
    /// prévia, confirmação e verificação por leitura.
    AlteraEstadoDoProcesso {
        o_que: EstadoDoProcesso,
    },
    Desconhecido,
}

/// Como o navegador envia o formulário, segundo a tela observada.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum EnvioDoFormulario {
    /// Botão `type=submit` com este nome participa do corpo.
    BotaoDeEnvio(&'static str),
    /// O JavaScript observado chama `submit()`: nenhum botão participa.
    SubmitPorScript,
    /// O script cria este oculto no fim (`appendChild`) e chama `submit()`
    /// (`selecionarUnidade()`, #205); o valor vem da operação.
    SubmitPorScriptComCampoCriado(&'static str),
    /// Botão `type=submit` com este nome é servido (exatamente um), mas o
    /// `onsubmit` observado o desabilita antes de o navegador montar o
    /// corpo: ele não participa.
    BotaoDesabilitadoPeloOnsubmit(&'static str),
}

/// Declaração `const` feita pela própria operação; não há catálogo central.
#[derive(Debug)]
pub(crate) struct OperacaoDeFormulario {
    pub(crate) acao: AcaoDoSei,
    pub(crate) formulario: &'static str,
    pub(crate) envio: EnvioDoFormulario,
    pub(crate) campos_ocultos_exigidos: &'static [&'static str],
    /// Ocultos que o JavaScript observado da operação reescreve; os demais
    /// saem exatamente como servidos.
    pub(crate) campos_ocultos_reescritos: &'static [&'static str],
    pub(crate) campos_usuario: &'static [&'static str],
    pub(crate) resultado: ResultadoDoEnvio,
    pub(crate) unidade: ConferenciaDaUnidade,
    pub(crate) efeito: EfeitoAvaliado,
}

/// Como a resposta de sucesso é reconhecida.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ResultadoDoEnvio {
    /// A resposta publica o marcador da operação; sem ele, recusa.
    Marcador(MarcadorDeResultado),
    /// A resposta publica pelo menos um destes marcadores; a operação
    /// distingue depois qual (tabela ou aviso de "sem resultado").
    UmDosMarcadores(&'static [MarcadorDeResultado]),
    /// Sem marcador observado: só HTML 200 sem tela de login. A operação
    /// é obrigada a conferir o efeito por leitura posterior.
    ConferidoPorLeituraPosterior,
}

/// Onde a unidade solicitada é conferida antes da derivação.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ConferenciaDaUnidade {
    /// `lnkInfraUnidade` único na tela do formulário, igual ao solicitado.
    NaTelaDoFormulario,
    /// Tela de janela sem barra da unidade, aberta por link de uma tela já
    /// conferida na mesma sessão; se publicar `lnkInfraUnidade`, também confere.
    NaTelaDeOrigemDoLink,
}

/// Elemento com classe que marca a resposta de sucesso da operação
/// (`<table class="pesquisaResultado">`), lido pelo tokenizador de atributos.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct MarcadorDeResultado {
    pub(crate) elemento: &'static str,
    pub(crate) classe: &'static str,
}

/// Charset declarado pela resposta que serviu a tela.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CodificacaoDaTela {
    Iso88591,
    Utf8,
}

/// HTML e URL exata da mesma resposta, na mesma geração de sessão.
pub(crate) struct TelaServida<'a> {
    pub(crate) url: &'a str,
    pub(crate) html: &'a str,
    pub(crate) codificacao: CodificacaoDaTela,
}

/// POST derivado; não é `Clone`, e o envio o consome (sem retry).
pub(crate) struct SubmissaoDerivada {
    url: String,
    host: String,
    caminho: String,
    acao: AcaoDaSubmissao,
    referer: String,
    nomes: Vec<String>,
    corpo: Vec<u8>,
    efeito: EfeitoAvaliado,
    resultado: ResultadoDoEnvio,
}

impl SubmissaoDerivada {
    pub(crate) const METODO: &'static str = "POST";
    pub(crate) const TIPO_DE_CONTEUDO: &'static str = "application/x-www-form-urlencoded";
    pub(crate) fn url(&self) -> &str {
        &self.url
    }
    pub(crate) fn origem(&self) -> String {
        format!("https://{}", self.host)
    }
    pub(crate) fn referer(&self) -> &str {
        &self.referer
    }
    pub(crate) fn nomes_dos_campos(&self) -> &[String] {
        &self.nomes
    }
    pub(crate) fn corpo(&self) -> &[u8] {
        &self.corpo
    }
    #[cfg(test)]
    pub(crate) fn efeito(&self) -> EfeitoAvaliado {
        self.efeito
    }
}

/// Só nomes e destino sem consulta: `infra_hash`, `Referer` e valores
/// nunca aparecem em logs ou mensagens de teste.
impl fmt::Debug for SubmissaoDerivada {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SubmissaoDerivada")
            .field("metodo", &Self::METODO)
            .field(
                "destino",
                &format_args!("https://{}/{}", self.host, self.caminho),
            )
            .field("acao", &self.acao)
            .field("campos", &self.nomes)
            .field("valores", &"<redigido>")
            .field("referer", &"<redigido>")
            .field("efeito", &self.efeito)
            .finish_non_exhaustive()
    }
}

fn ordenar_campos<'c>(
    controles: &'c [Controle],
    operacao: &OperacaoDeFormulario,
    campos: &'c [(&'c str, &'c str)],
) -> Result<Vec<(&'c str, &'c str)>, ErroSubmissaoFormulario> {
    let criado = match operacao.envio {
        EnvioDoFormulario::SubmitPorScriptComCampoCriado(campo) => Some(campo),
        EnvioDoFormulario::BotaoDeEnvio(_)
        | EnvioDoFormulario::SubmitPorScript
        | EnvioDoFormulario::BotaoDesabilitadoPeloOnsubmit(_) => None,
    };
    for (i, (nome, _)) in campos.iter().enumerate() {
        let campo = (*nome).to_owned();
        if criado == Some(*nome) {
            if controles.iter().any(|c| c.nome() == *nome) {
                return Err(ErroSubmissaoFormulario::CampoCriadoPeloScriptJaPublicado { campo });
            }
            if campos.iter().take(i).any(|(anterior, _)| anterior == nome) {
                return Err(ErroSubmissaoFormulario::CampoInformadoMaisDeUmaVez { campo });
            }
            continue;
        }
        let oculto = operacao.campos_ocultos_reescritos.contains(nome);
        let usuario = operacao.campos_usuario.iter().any(|permitido| {
            *permitido == *nome
                || permitido
                    .strip_suffix('*')
                    .is_some_and(|prefixo| nome.starts_with(prefixo))
        });
        if !oculto && !usuario {
            return Err(ErroSubmissaoFormulario::CampoForaDaListaDaOperacao { campo });
        }
        let publicados: Vec<_> = controles
            .iter()
            .filter(|c| match c {
                Controle::Oculto { nome: n, .. } => oculto && n == nome,
                Controle::Usuario(n) | Controle::SelecaoMultipla(n) => !oculto && n == nome,
                Controle::Botao { .. } => false,
            })
            .collect();
        if publicados.is_empty() {
            return Err(ErroSubmissaoFormulario::CampoNaoPublicadoPeloFormulario { campo });
        }
        // Só um `<select multiple>` (e nenhum outro controle de mesmo nome)
        // admite o nome repetido; os valores vêm da operação.
        let multipla = publicados
            .iter()
            .all(|c| matches!(c, Controle::SelecaoMultipla(_)));
        if !multipla && campos.iter().take(i).any(|(anterior, _)| anterior == nome) {
            return Err(ErroSubmissaoFormulario::CampoInformadoMaisDeUmaVez { campo });
        }
    }
    // Ordem do documento; um rádio com vários controles é emitido uma vez,
    // e os pares de um `select multiple` saem juntos na posição dele.
    let mut emitidos: Vec<&str> = Vec::new();
    let mut ordenados = Vec::new();
    for controle in controles {
        match controle {
            Controle::Oculto { nome, valor } => {
                // O valor reescrito pela regra da operação ocupa a posição servida.
                let reescrito = operacao
                    .campos_ocultos_reescritos
                    .contains(&nome.as_str())
                    .then(|| campos.iter().find(|(n, _)| n == nome))
                    .flatten();
                ordenados.push(
                    reescrito
                        .copied()
                        .unwrap_or((nome.as_str(), valor.as_str())),
                );
            }
            Controle::Botao { nome, valor } => ordenados.push((nome.as_str(), valor.as_str())),
            Controle::Usuario(nome) | Controle::SelecaoMultipla(nome)
                if !emitidos.contains(&nome.as_str()) =>
            {
                emitidos.push(nome);
                ordenados.extend(campos.iter().filter(|(n, _)| n == nome).copied());
            }
            Controle::Usuario(_) | Controle::SelecaoMultipla(_) => {}
        }
    }
    // `appendChild` antes do `submit()`: o campo criado é o último.
    if let Some(campo) = criado {
        ordenados.extend(campos.iter().filter(|(n, _)| *n == campo).copied());
    }
    Ok(ordenados)
}

/// Corpo `application/x-www-form-urlencoded` no charset da tela.
fn codificar_corpo(
    ordenados: &[(&str, &str)],
    codificacao: CodificacaoDaTela,
) -> Result<String, ErroSubmissaoFormulario> {
    let mut corpo = String::new();
    for (i, (nome, valor)) in ordenados.iter().enumerate() {
        if i > 0 {
            corpo.push('&');
        }
        let nao_representavel = || ErroSubmissaoFormulario::ValorNaoRepresentavelNaCodificacao {
            campo: (*nome).to_owned(),
        };
        codificar(nome, codificacao, &mut corpo).ok_or_else(nao_representavel)?;
        corpo.push('=');
        codificar(valor, codificacao, &mut corpo).ok_or_else(nao_representavel)?;
    }
    Ok(corpo)
}

/// Deriva a submissão do formulário servido, sem rede.
pub(crate) fn derivar_submissao(
    operacao: &OperacaoDeFormulario,
    instalacao: &Instalacao,
    tela: &TelaServida<'_>,
    unidade_solicitada: &str,
    campos: &[(&str, &str)],
) -> Result<SubmissaoDerivada, ErroSubmissaoFormulario> {
    derivar_submissao_com_action(operacao, instalacao, tela, unidade_solicitada, campos, None)
}

/// Somente para action reescrita por handler literal observado da mesma tela.
/// A URL declarada atravessa a mesma validação de host, caminho, query e hash.
#[allow(clippy::too_many_arguments)] // O parâmetro adicional é a action observada.
pub(crate) fn derivar_submissao_com_action(
    operacao: &OperacaoDeFormulario,
    instalacao: &Instalacao,
    tela: &TelaServida<'_>,
    unidade_solicitada: &str,
    campos: &[(&str, &str)],
    url_declarada: Option<&str>,
) -> Result<SubmissaoDerivada, ErroSubmissaoFormulario> {
    if operacao.efeito == EfeitoAvaliado::Desconhecido {
        return Err(ErroSubmissaoFormulario::EfeitoDaOperacaoNaoAvaliado);
    }
    if tela.html.len() > crate::MAX_HTML_BYTES {
        return Err(ErroSubmissaoFormulario::Transporte(
            FalhaTransporte::RespostaExcessiva,
        ));
    }
    let servida = UrlHttps::interpretar(tela.url).ok_or(ErroSubmissaoFormulario::Transporte(
        FalhaTransporte::UrlInvalida,
    ))?;
    if !instalacao.autoriza_controlador(servida.host, servida.caminho) {
        return Err(ErroSubmissaoFormulario::Transporte(
            FalhaTransporte::OrigemNaoAutorizada,
        ));
    }
    // tl só recusa entradas acima de u32::MAX bytes, já barradas acima.
    let dom = tl::parse(tela.html, tl::ParserOptions::default())
        .map_err(ErroSubmissaoFormulario::HtmlDaTelaNaoInterpretado)?;
    match operacao.unidade {
        ConferenciaDaUnidade::NaTelaDoFormulario => conferir_unidade(&dom, unidade_solicitada)?,
        ConferenciaDaUnidade::NaTelaDeOrigemDoLink => {
            if !nos_com_id(&dom, "lnkInfraUnidade")?.is_empty() {
                conferir_unidade(&dom, unidade_solicitada)?;
            }
        }
    }
    let form = localizar_formulario(&dom, operacao.formulario)?;
    conferir_metodo_e_codificacao(form)?;
    let url = match url_declarada {
        Some(url) => resolver_action_declarada(url, &servida, instalacao, operacao)?,
        None => resolver_action(form, &servida, instalacao, operacao)?,
    };
    let controles = ler_controles(form, dom.parser(), operacao)?;
    let ordenados = ordenar_campos(&controles, operacao, campos)?;
    let corpo = codificar_corpo(&ordenados, tela.codificacao)?;
    Ok(SubmissaoDerivada {
        url,
        host: servida.host.to_owned(),
        caminho: servida.caminho.to_owned(),
        acao: AcaoDaSubmissao::Controlador(operacao.acao),
        referer: tela.url.to_owned(),
        nomes: ordenados.iter().map(|(n, _)| (*n).to_owned()).collect(),
        corpo: corpo.into_bytes(),
        efeito: operacao.efeito,
        resultado: operacao.resultado,
    })
}

/// Resposta do POST: a mesma `RespostaWeb` do GET mais o cabeçalho
/// `location`, que só o envio de formulário lê. O GET do Controle não muda.
pub(crate) struct RespostaDoPost<R: Read> {
    pub(crate) resposta: RespostaWeb<R>,
    pub(crate) localizacao: Option<String>,
}

/// 302 recebido e **não seguido**; o corpo não é lido. `Location` e URL do
/// POST são assinados: sem `Clone`, e o `Debug` os redige.
pub(crate) struct Redirecionamento302 {
    localizacao: Option<String>,
    url_do_envio: String,
}

impl Redirecionamento302 {
    pub(crate) fn localizacao(&self) -> Option<&str> {
        self.localizacao.as_deref()
    }
    pub(crate) fn url_do_envio(&self) -> &str {
        &self.url_do_envio
    }
    #[cfg(test)]
    pub(crate) fn sintetico(localizacao: Option<&str>, url_do_envio: &str) -> Self {
        Self {
            localizacao: localizacao.map(str::to_owned),
            url_do_envio: url_do_envio.to_owned(),
        }
    }
}

impl fmt::Debug for Redirecionamento302 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Redirecionamento302")
            .field(
                "localizacao",
                &self.localizacao.as_ref().map(|_| "<redigido>"),
            )
            .field("url_do_envio", &"<redigido>")
            .finish()
    }
}

/// HTML com o marcador da operação, ou 302 entregue à operação como dado.
#[derive(Debug)]
pub(crate) enum RespostaDoEnvio {
    Html(String),
    Redirecionamento(Redirecionamento302),
}

/// Envio injetável: `FnOnce` recebe (e consome) a submissão e o cookie,
/// uma única vez. A submissão já traz o destino autorizado pela derivação.
/// Só se lê o cookie privado, pois o destino pode vir de uma tela intermediária.
/// Nenhum 3xx é seguido; só o 302 volta como dado, para a operação interpretar.
pub(crate) fn enviar_submissao_via<R: Read, F>(
    submissao: SubmissaoDerivada,
    cookie: &CookieDaSessao,
    enviar: F,
) -> Result<RespostaDoEnvio, ErroSubmissaoFormulario>
where
    F: FnOnce(SubmissaoDerivada, &str) -> Result<RespostaDoPost<R>, FalhaTransporte>,
{
    let cookie = cookie.valor()?;
    let resultado = submissao.resultado;
    let url_do_envio = submissao.url.clone();
    let RespostaDoPost {
        resposta,
        localizacao,
    } = enviar(submissao, cookie)?;
    if resposta.status_code == 302 {
        // Corpo descartado sem leitura; a operação decide o que o 302 significa.
        return Ok(RespostaDoEnvio::Redirecionamento(Redirecionamento302 {
            localizacao,
            url_do_envio,
        }));
    }
    let html = decodificar_resposta(resposta).map_err(ErroSubmissaoFormulario::da_resposta)?;
    // 200 com a validação da Infra: recusa do SEI, com ou sem marcador.
    if let Some(mensagem) = mensagem_de_validacao_da_infra(&html) {
        return Err(ErroSubmissaoFormulario::ValidacaoRecusadaPeloSei(mensagem));
    }
    let marcadores: &[MarcadorDeResultado] = match &resultado {
        ResultadoDoEnvio::ConferidoPorLeituraPosterior => &[],
        ResultadoDoEnvio::Marcador(marcador) => std::slice::from_ref(marcador),
        ResultadoDoEnvio::UmDosMarcadores(marcadores) => marcadores,
    };
    let mut marcado = resultado == ResultadoDoEnvio::ConferidoPorLeituraPosterior;
    if !marcadores.is_empty() {
        let dom = tl::parse(&html, tl::ParserOptions::default())
            .map_err(ErroSubmissaoFormulario::HtmlDaTelaNaoInterpretado)?;
        for MarcadorDeResultado { elemento, classe } in marcadores {
            marcado |= !nos_por_elemento_e_classe(&dom, None, elemento, classe)?.is_empty();
        }
    }
    if !marcado {
        return Err(ErroSubmissaoFormulario::MarcadorDeResultadoAusenteNaResposta);
    }
    Ok(RespostaDoEnvio::Html(html))
}

#[cfg(test)]
#[path = "formulario_web/tests.rs"]
pub(crate) mod tests;

#[cfg(test)]
#[path = "formulario_web/tests_envio.rs"]
mod tests_envio;

#[cfg(test)]
#[path = "formulario_web/tests_selecao.rs"]
mod tests_selecao;

#[cfg(test)]
#[path = "formulario_web/tests_atributos.rs"]
mod tests_atributos;

#[cfg(test)]
#[path = "formulario_web/tests_reescrita.rs"]
mod tests_reescrita;

#[cfg(test)]
#[path = "formulario_web/tests_conferencia.rs"]
mod tests_conferencia;

#[cfg(test)]
#[path = "formulario_web/tests_login.rs"]
mod tests_login;

#[cfg(test)]
#[path = "formulario_web/tests_campo_criado.rs"]
mod tests_campo_criado;
