//! Orquestração de `ies login`: lock, conferência da sessão gravada (um
//! GET), login pela raiz da instalação e gravação da nova geração.
use super::cookies::PoteDeCookies;
use super::gravacao::{self, TravaDoLogin};
use super::redirecionamento::resolver_no_host;
use super::{
    Conexao, Credenciais, ErroLogin, MAX_REDIRECIONAMENTOS, PedidoDeLogin, RespostaHttp,
    ResultadoDoLogin, TransporteDeLogin, tela,
};
use crate::erro_web::ErroCaixa;
use crate::formulario_web::{SubmissaoDerivada, TelaServida};
use crate::indice_local::IndiceLocal;
use crate::instalacao::Instalacao;
use crate::transporte::contagem::ContagemDeRequisicoes;
use crate::transporte::{self, FalhaTransporte, RespostaWeb};
use std::io::Cursor;

/// Conta cada pedido feito, sem alterar nenhum.
struct Contador<'t, T> {
    transporte: &'t mut T,
    pedidos: ContagemDeRequisicoes,
}

impl<T: TransporteDeLogin> TransporteDeLogin for Contador<'_, T> {
    fn get(&mut self, url: &str, cookie: &str) -> Result<RespostaHttp, FalhaTransporte> {
        self.pedidos.registrar();
        self.transporte.get(url, cookie)
    }
    fn post(
        &mut self,
        submissao: SubmissaoDerivada,
        cookie: &str,
    ) -> Result<RespostaHttp, FalhaTransporte> {
        self.pedidos.registrar();
        self.transporte.post(submissao, cookie)
    }
}

/// Sessão obtida; nunca é impressa.
struct SessaoObtida {
    url: String,
    cookie: String,
    unidade: String,
    controle: ControleObtido,
}

/// HTML e `Content-Type` do Controle já lido, para a colheita do índice.
struct ControleObtido {
    html: crate::tela_html::HtmlServido,
    mime: String,
}

/// A sessão gravada que respondeu com o Controle.
struct SessaoGravada {
    url: String,
    cookie: String,
    unidade: String,
    controle: ControleObtido,
}

fn como_web(resposta: RespostaHttp) -> RespostaWeb<Cursor<Vec<u8>>> {
    RespostaWeb {
        status_code: i32::from(resposta.status),
        mime: resposta
            .cabecalho("content-type")
            .unwrap_or_default()
            .to_owned(),
        content_length: None,
        localizacao: resposta.cabecalho("location").map(str::to_owned),
        corpo: Cursor::new(resposta.corpo),
    }
}

/// Unidade do Controle, pela mesma leitura da caixa (duas tabelas e
/// `lnkInfraUnidade` único).
fn unidade_do_controle(html: &crate::tela_html::HtmlServido) -> Result<String, ErroCaixa> {
    // A conta pode abrir na visão detalhada herdada do último uso (SEI 5.x).
    crate::controle_reconhecido::unidade_do_controle(html)
}

/// Exceção inventariada (#183): o login consulta URL e cookie como valores
/// opcionais antes da rede para distinguir sessão ausente, vazia ou expirada
/// e conferir o marcador. Uma leitura comum de `ReferenciasDaSessao` recusaria
/// ausentes/vazios antes dessa decisão; a coerência entre as duas leituras
/// vem do lock, que os escritores também pegam (#222). A checagem
/// 0600/sem symlink continua feita pelo primitivo único `arquivo_privado`.
///
/// Um arquivo da sessão gravada; ausente ou vazio vale "sem sessão".
/// O login precisa ler URL e cookie para conferir a geração antes de
/// validar a sintaxe. Arquivos de gerações diferentes valem sessão ausente.
fn ler_da_sessao(caminho: &std::path::Path) -> Result<Option<String>, ErroLogin> {
    match transporte::arquivo_privado(caminho, 8_192) {
        Ok(texto) => Ok(Some(texto.trim().to_owned())),
        // Vazio: o arquivo que o `ies setup` exige antes do primeiro login.
        Err(
            FalhaTransporte::ArquivoPrivadoAusenteOuInacessivel(_)
            | FalhaTransporte::ArquivoPrivadoVazio,
        ) => Ok(None),
        Err(falha) => Err(SessaoGravadaInvalida::Arquivo(falha).into()),
    }
}

/// Um GET da sessão gravada. `Some`: válida. `None`: ausente, de outra
/// geração ou vencida (tela de login, 3xx, 401/403).
fn conferir_sessao_gravada<T: TransporteDeLogin>(
    pedido: &PedidoDeLogin<'_>,
    origem: &str,
    transporte: &mut T,
) -> Result<Option<SessaoGravada>, ErroLogin> {
    let arquivos = pedido.arquivos;
    let Some(url) = ler_da_sessao(&arquivos.url_controle)? else {
        return Ok(None);
    };
    let Some(cookie) = ler_da_sessao(&arquivos.cookie)? else {
        return Ok(None);
    };
    if !gravacao::mesma_geracao(arquivos, &url, &cookie) {
        return Ok(None);
    }
    // Mesmas travas do GET da caixa, antes de qualquer conexão.
    transporte::validar_origem(&url, origem).map_err(SessaoGravadaInvalida::Arquivo)?;
    transporte::validar_cookie(&cookie).map_err(SessaoGravadaInvalida::Arquivo)?;
    let resposta = transporte.get(&url, &cookie)?;
    let web = como_web(resposta);
    let mime = web.mime.clone();
    match transporte::decodificar_resposta(web) {
        Ok(html) => {
            let unidade = unidade_do_controle(&html)
                .map_err(ErroLogin::ControleDaSessaoExistenteNaoReconhecido)?;
            Ok(Some(SessaoGravada {
                url,
                cookie,
                unidade,
                controle: ControleObtido { html, mime },
            }))
        }
        Err(
            FalhaTransporte::PaginaDeLoginRecebidaNoControle
            // 3xx e 401/403: a sessão gravada não vale mais.
            | FalhaTransporte::Status(crate::transporte::StatusHttpRecusado {
                status: 300..=399 | 401 | 403,
                ..
            }),
        ) => Ok(None),
        Err(falha) => Err(ErroLogin::Transporte(falha)),
    }
}

/// GETs a partir de `url` seguindo até 8 redirecionamentos no host;
/// devolve a URL e a resposta final (não 3xx).
fn navegar<T: TransporteDeLogin>(
    transporte: &mut T,
    pote: &mut PoteDeCookies,
    host: &str,
    url: String,
) -> Result<(String, RespostaHttp), ErroLogin> {
    let mut atual = url;
    let mut saltos = 0;
    loop {
        let resposta = transporte.get(&atual, &pote.cabecalho(&atual))?;
        pote.absorver(&atual, &resposta)?;
        if !(300..400).contains(&resposta.status) {
            return Ok((atual, resposta));
        }
        saltos += 1;
        if saltos > MAX_REDIRECIONAMENTOS {
            return Err(ErroLogin::RedirecionamentosAcimaDoLimite);
        }
        atual = proximo(&atual, &resposta, host)?;
    }
}

fn proximo(atual: &str, resposta: &RespostaHttp, host: &str) -> Result<String, ErroLogin> {
    let localizacao = resposta
        .cabecalho("location")
        .ok_or(ErroLogin::RedirecionamentoSemLocalizacao)?;
    resolver_no_host(atual, localizacao, host)
}

/// HTML e `Content-Type` de uma resposta 200; outro status vira a recusa
/// do transporte (com status e mensagem do SEI).
fn html_200(resposta: RespostaHttp) -> Result<(crate::tela_html::HtmlServido, String), ErroLogin> {
    let web = como_web(resposta);
    let mime = web.mime.clone();
    if web.status_code != 200 {
        return Err(ErroLogin::Transporte(transporte::recusar_status(web)));
    }
    if !crate::mime_web::e_html(&mime) {
        return Err(ErroLogin::Transporte(
            FalhaTransporte::TipoDeConteudoNaoHtml,
        ));
    }
    let html =
        crate::tela_html::HtmlServido::do_corpo(web.corpo.get_ref(), &mime).map_err(|erro| {
            ErroLogin::Transporte(FalhaTransporte::CorpoNaoDecodificavelNoCharsetDeclarado(
                erro,
            ))
        })?;
    Ok((html, mime))
}

/// Controle de Processos na URL assinada da instalação, com cookie.
/// `None` se a URL não é a do Controle.
fn como_controle(
    url: &str,
    (html, mime): (&crate::tela_html::HtmlServido, &str),
    origem: &str,
    pote: &PoteDeCookies,
) -> Option<Result<SessaoObtida, ErroLogin>> {
    transporte::validar_origem(url, origem).ok()?;
    Some(
        unidade_do_controle(html)
            .map_err(ErroLogin::ControleAposOLoginNaoReconhecido)
            .and_then(|unidade| {
                let cookie = pote.cabecalho(url);
                if cookie.is_empty() || transporte::validar_cookie(&cookie).is_err() {
                    return Err(ErroLogin::CookieDaSessaoNaoRecebido);
                }
                Ok(SessaoObtida {
                    url: url.to_owned(),
                    cookie,
                    unidade,
                    controle: ControleObtido {
                        html: html.to_owned(),
                        mime: mime.to_owned(),
                    },
                })
            }),
    )
}

/// Tela de login servida pela raiz, antes de qualquer credencial.
fn tela_inicial<T: TransporteDeLogin>(
    transporte: &mut T,
    pote: &mut PoteDeCookies,
    host: &str,
) -> Result<(String, crate::tela_html::HtmlServido, String), ErroLogin> {
    let (url, resposta) = navegar(transporte, pote, host, format!("https://{host}/"))?;
    let (html, mime) = html_200(resposta)?;
    if tela::captcha_exigido(&html) {
        return Err(ErroLogin::CaptchaExigido);
    }
    if tela::segundo_fator_exigido(&html) {
        return Err(ErroLogin::SegundoFatorExigido);
    }
    if !crate::tela_login::e_tela_login(&html) {
        return Err(ErroLogin::TelaDeLoginNaoReconhecida);
    }
    Ok((url, html, mime))
}

/// Login pela raiz: tela, um POST e a navegação até o Controle.
fn autenticar<T: TransporteDeLogin>(
    instalacao: &Instalacao,
    origem: &str,
    credenciais: &Credenciais,
    transporte: &mut T,
) -> Result<SessaoObtida, ErroLogin> {
    let host = instalacao.host();
    let mut pote = PoteDeCookies::novo(host);
    let (url_da_tela, html, mime) = tela_inicial(transporte, &mut pote, host)?;
    let codificacao = crate::pagina_do_controle::codificacao_da_tela(&mime)
        .ok_or(ErroLogin::CharsetDaTelaDeLoginNaoSuportado)?;
    let servida = TelaServida {
        url: &url_da_tela,
        html: &html,
        codificacao,
    };
    let submissao = tela::derivar_login(instalacao, &servida, credenciais)?;
    let url_do_post = submissao.url().to_owned();
    let cookie = pote.cabecalho(&url_do_post);
    let resposta = transporte.post(submissao, &cookie)?;
    pote.absorver(&url_do_post, &resposta)?;
    let (url, resposta) = match resposta.status {
        // Seguir repetiria o corpo com as credenciais.
        307 | 308 => return Err(ErroLogin::RedirecionamentoQueRepetiriaOEnvioDoLogin),
        300..=399 => {
            let seguinte = proximo(&url_do_post, &resposta, host)?;
            navegar(transporte, &mut pote, host, seguinte)?
        }
        _ => (url_do_post, resposta),
    };
    let (html, mime) = html_200(resposta)?;
    if let Some(obtida) = como_controle(&url, (&html, &mime), origem, &pote) {
        return obtida;
    }
    if tela::captcha_exigido(&html) {
        return Err(ErroLogin::CaptchaExigido);
    }
    if tela::segundo_fator_exigido(&html) {
        return Err(ErroLogin::SegundoFatorExigido);
    }
    if crate::tela_login::e_tela_login(&html) {
        return Err(ErroLogin::CredencialRecusada {
            mensagem_do_servidor: tela::mensagem_de_recusa(&html),
        });
    }
    // Página inicial da conta (Painel de Controle): só o link do menu.
    let link = tela::link_do_controle(&html)?.ok_or(ErroLogin::TelaAposOLoginNaoReconhecida)?;
    let destino = resolver_no_host(&url, &link, host)?;
    transporte::validar_origem(&destino, origem).map_err(ErroLogin::Transporte)?;
    let (url, resposta) = navegar(transporte, &mut pote, host, destino)?;
    let (html, mime) = html_200(resposta)?;
    como_controle(&url, (&html, &mime), origem, &pote).unwrap_or(Err(
        ErroLogin::ControleAposOLoginNaoReconhecido(
            ErroCaixa::IdentificacaoDaUnidadeAusenteNoControle,
        ),
    ))
}

fn conferir_unidade(pedido: &PedidoDeLogin<'_>, unidade: &str) -> Result<(), ErroLogin> {
    if pedido
        .unidade_esperada
        .is_some_and(|esperada| esperada != unidade)
    {
        return Err(ErroLogin::UnidadeExibidaDiferenteDaSolicitada);
    }
    Ok(())
}

/// Fluxo completo com transporte injetável; conta cada pedido feito. Sem
/// índice local (a CLI usa [`entrar_colhendo`]).
#[cfg(test)]
pub(crate) fn entrar_via<T: TransporteDeLogin>(
    pedido: &PedidoDeLogin<'_>,
    transporte: &mut T,
) -> Result<ResultadoDoLogin, ErroLogin> {
    entrar_colhendo(pedido, &IndiceLocal::desligado(), transporte)
}

/// [`entrar_via`] e, com a sessão válida ou recém-gravada, a colheita do
/// Controle já obtido no índice local (nenhuma requisição a mais).
pub(crate) fn entrar_colhendo<T: TransporteDeLogin>(
    pedido: &PedidoDeLogin<'_>,
    indice: &IndiceLocal,
    transporte: &mut T,
) -> Result<ResultadoDoLogin, ErroLogin> {
    // Exceção inventariada (#183): o login de origem não dispõe ainda da
    // URL do Controle e precisa primeiro selecionar a instalação pelo arquivo.
    let origem = transporte::arquivo_privado(&pedido.arquivos.origem, 512)
        .map_err(OrigemIlegivel::Arquivo)?;
    let origem = origem.trim();
    let instalacao = Instalacao::interpretar(origem).ok_or(OrigemIlegivel::Arquivo(
        FalhaTransporte::OrigemNaoAutorizada,
    ))?;
    let _trava = TravaDoLogin::obter(&pedido.arquivos.url_controle, pedido.espera_do_lock)?;
    let mut contador = Contador {
        transporte,
        pedidos: ContagemDeRequisicoes::default(),
    };
    if !pedido.forcar
        && let Some(gravada) = conferir_sessao_gravada(pedido, origem, &mut contador)?
    {
        conferir_unidade(pedido, &gravada.unidade)?;
        colher(
            indice,
            &instalacao,
            (&gravada.cookie, &gravada.url),
            &gravada.controle,
        );
        return Ok(ResultadoDoLogin {
            instalacao: instalacao.url_base(),
            unidade_exibida: gravada.unidade,
            conexao: Conexao::SessaoExistenteValida,
            requisicoes: contador.pedidos.total(),
        });
    }
    let unidade = logar_e_gravar(pedido, &instalacao, origem, indice, &mut contador)?;
    Ok(ResultadoDoLogin {
        instalacao: instalacao.url_base(),
        unidade_exibida: unidade,
        conexao: Conexao::LoginRealizado,
        requisicoes: contador.pedidos.total(),
    })
}

/// Credenciais, login, unidade conferida, gravação da nova geração e
/// colheita do Controle obtido; devolve a unidade. Com o lock obtido.
fn logar_e_gravar<T: TransporteDeLogin>(
    pedido: &PedidoDeLogin<'_>,
    instalacao: &Instalacao,
    origem: &str,
    indice: &IndiceLocal,
    transporte: &mut T,
) -> Result<String, ErroLogin> {
    let caminho = pedido
        .credenciais
        .ok_or(ErroLogin::CaminhoDasCredenciaisNaoInformado)?;
    let credenciais = Credenciais::ler(caminho)?;
    let obtida = autenticar(instalacao, origem, &credenciais, transporte)?;
    conferir_unidade(pedido, &obtida.unidade)?;
    gravacao::gravar(
        pedido.arquivos,
        &instalacao.url_base(),
        &obtida.url,
        &obtida.cookie,
    )?;
    // A chave é a do cookie e da URL que acabaram de ser gravados.
    colher(
        indice,
        instalacao,
        (&obtida.cookie, &obtida.url),
        &obtida.controle,
    );
    Ok(obtida.unidade)
}

/// Desfecho do relogin de uma leitura.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Relogado {
    LoginRealizado,
    /// Outro processo já gravou outra geração: nenhum login.
    GeracaoJaTrocada,
}

/// Relogin de uma leitura que achou a sessão encerrada: sob o lock, a
/// geração gravada é comparada com `inicial` (a do início do comando);
/// se mudou, outro processo já relogou e nada é pedido. Senão, o login
/// pela raiz, sem o GET de conferência (a leitura já viu a sessão vencida).
pub(super) fn relogar<T: TransporteDeLogin>(
    pedido: &PedidoDeLogin<'_>,
    inicial: Option<&str>,
    indice: &IndiceLocal,
    transporte: &mut T,
) -> Result<Relogado, ErroLogin> {
    let origem = transporte::arquivo_privado(&pedido.arquivos.origem, 512)
        .map_err(OrigemIlegivel::Arquivo)?;
    let origem = origem.trim();
    let instalacao = Instalacao::interpretar(origem).ok_or(OrigemIlegivel::Arquivo(
        FalhaTransporte::OrigemNaoAutorizada,
    ))?;
    let _trava = TravaDoLogin::obter(&pedido.arquivos.url_controle, pedido.espera_do_lock)?;
    if gravacao::geracao_gravada(pedido.arquivos).as_deref() != inicial {
        return Ok(Relogado::GeracaoJaTrocada);
    }
    logar_e_gravar(pedido, &instalacao, origem, indice, transporte)?;
    Ok(Relogado::LoginRealizado)
}

/// Colheita do Controle já lido (nenhum pedido); nunca falha o login.
fn colher(
    indice: &IndiceLocal,
    instalacao: &Instalacao,
    (cookie, url): (&str, &str),
    controle: &ControleObtido,
) {
    crate::colheita_do_controle::Colheita::do_cookie(indice, instalacao, cookie, url)
        .colher(&controle.html, Some(&controle.mime));
}

/// O arquivo de origem da instalação ausente, inseguro ou inválido. Tipo próprio da etapa; o nome
/// público é o motivo do transporte (`transparente`).
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum OrigemIlegivel {
    Arquivo(#[from] crate::transporte::FalhaTransporte),
}

/// URL ou cookie gravados que o transporte recusa antes da rede. Tipo próprio da etapa; o nome
/// público é o motivo do transporte (`transparente`).
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum SessaoGravadaInvalida {
    Arquivo(#[from] crate::transporte::FalhaTransporte),
}
