//! Apoio dos testes do fluxo: transporte roteirizado, índice em diretório
//! temporário e telas SINTÉTICAS com a estrutura observada (Controle com
//! `tr#P{id}` e `frmProtocoloPesquisaRapida`, 302 da pesquisa rápida de GO,
//! tela do processo e árvore). Hosts, hashes, ids e números inventados.
use super::*;
use crate::acao_do_sei::AcaoDoSei;
use crate::arvore_processo::tests::{NUP, arvore, no_documento, no_processo};
use crate::arvore_processo::tests_tela::{src_da_arvore, tela_do_processo};
use crate::formulario_web::SubmissaoDerivada;
pub(super) use crate::historico_processo::tests_apoio::corpo_ver_completo;
use crate::historico_processo::tests_apoio::{
    HASH_HISTORICO, action_do_historico, com_link, forma_go, forma_unb, link_do_historico,
    pagina_do_historico,
};
use crate::indice_local::{ChaveDoLink, IndiceLocal};
use crate::instalacao::Instalacao;
use crate::transporte::RespostaWeb;
use crate::transporte::sessao::CookieDaSessao;
use std::collections::VecDeque;
use std::fmt::Write as _;
use std::io::Cursor;
use std::path::PathBuf;

pub(super) const ORIGEM: &str = "https://sei.exemplo.gov.br/sei";
pub(super) const UNIDADE: &str = "UNIDADE TESTE";
pub(super) const ID_UNIDADE: &str = "110000001";
pub(super) const ID: &str = "10";
const CONTEXTO: &str = "infra_sistema=100000100&infra_unidade_atual=110000001";
pub(super) const COOKIE: &str = "PHPSESSID=SINTETICO";

pub(super) fn hash(c: char) -> String {
    c.to_string().repeat(64)
}

pub(super) fn instalacao() -> Instalacao {
    Instalacao::interpretar(ORIGEM).unwrap()
}

pub(super) fn url_controle() -> String {
    format!(
        "{ORIGEM}/controlador.php?acao=procedimento_controlar&{CONTEXTO}&infra_hash={}",
        hash('a')
    )
}

pub(super) fn action_rapida() -> String {
    format!(
        "controlador.php?acao=protocolo_pesquisa_rapida&{CONTEXTO}&infra_hash={}",
        hash('b')
    )
}

/// `Location` relativo do 302 da pesquisa rápida de NUP (GO).
pub(super) fn trabalhar_rapida(id: &str) -> String {
    trabalhar_rapida_com("id_protocolo", id)
}

/// Pesquisa pelo número SEI: forma `id_documento` das sete cassettes.
pub(super) fn trabalhar_rapida_documento(id: &str) -> String {
    trabalhar_rapida_com("id_documento", id)
}

fn trabalhar_rapida_com(chave: &str, id: &str) -> String {
    format!(
        "controlador.php?acao=procedimento_trabalhar&acao_origem=protocolo_pesquisa_rapida&{chave}={id}&{CONTEXTO}&infra_hash={}",
        hash('c')
    )
}

/// `href` do link da linha `tr#P{id}` no Controle.
pub(super) fn trabalhar_controle(id: &str) -> String {
    format!(
        "controlador.php?acao=procedimento_trabalhar&acao_origem=procedimento_controlar&acao_retorno=procedimento_controlar&id_procedimento={id}&{CONTEXTO}&infra_hash={}",
        hash('d')
    )
}

/// `Location` da entrada por id aceita (ADR 0003: sem `acao_origem`).
pub(super) fn trabalhar_entrada(id: &str) -> String {
    format!(
        "{ORIGEM}/controlador.php?acao=procedimento_trabalhar&id_procedimento={id}&infra_hash={}&{CONTEXTO}",
        hash('9')
    )
}

pub(super) fn url_entrada(id: &str) -> String {
    format!("{ORIGEM}/controlador.php?acao=procedimento_trabalhar&id_procedimento={id}")
}

pub(super) fn absoluta(relativa: &str) -> String {
    format!("{ORIGEM}/{relativa}")
}

pub(super) fn url_arvore(id: &str) -> String {
    absoluta(&src_da_arvore(id))
}

pub(super) const LOGIN_RECUSA: &str = "https://sei.exemplo.gov.br/sip/login.php?sigla_orgao_sistema=ORG&sigla_sistema=SEI&msg=Link%20sem%20assinatura";
pub(super) const LOGIN: &str =
    "https://sei.exemplo.gov.br/sip/login.php?sigla_orgao_sistema=ORG&sigla_sistema=SEI";

pub(super) fn formulario_rapido() -> String {
    format!(
        "<form class=\"form-inline\" id=\"frmProtocoloPesquisaRapida\" method=\"post\" action=\"{}\">\
         <div class=\"input-group\"><input type=\"text\" id=\"txtPesquisaRapida\" name=\"txtPesquisaRapida\" class=\"form-control\" placeholder=\"Pesquisar...\" tabindex=\"52\" />\
         <span class=\"input-group-btn\"><span id=\"spnInfraUnidade\" class=\"btn\">\
         <img src=\"svg/pesquisa_rapida.svg\" onclick=\"document.getElementById('frmProtocoloPesquisaRapida').submit();\" /></span></span></div></form>",
        action_rapida()
    )
}

/// Controle na visão resumida com `linhas` (id, número) em Recebidos.
pub(super) fn controle(linhas: &[(&str, &str)]) -> String {
    let mut tabela = String::new();
    for (id, numero) in linhas {
        let _ = write!(
            tabela,
            "<tr id=\"P{id}\"><td><a href=\"{}\" class=\"processoVisualizado\">{numero}</a></td></tr>",
            trabalhar_controle(id)
        );
    }
    let n = linhas.len();
    let legenda = if n == 0 {
        "0 registros:".to_owned()
    } else {
        format!("Processos recebidos ({n} registros - 1 a {n}):")
    };
    format!(
        "<html><body>{}<a id=\"lnkInfraUnidade\">{UNIDADE}</a>\
         <table id=\"tblProcessosRecebidos\"><caption>{legenda}</caption>{tabela}</table>\
         <table id=\"tblProcessosGerados\"><caption>0 registros:</caption></table></body></html>",
        formulario_rapido()
    )
}

/// Tela do processo com a árvore do id e o formulário rápido.
pub(super) fn trabalhar(id: &str) -> String {
    tela_do_processo(UNIDADE, &[src_da_arvore(id)])
        .replace("</body>", &format!("{}</body>", formulario_rapido()))
}

pub(super) fn arvore_de(id: &str, nup: &str) -> String {
    arvore(&[
        no_processo(0, id, nup),
        no_documento(1, "21", id, "0000101"),
    ])
}

/// Árvore com o link do histórico na forma de GO (`onclick` com a URL).
pub(super) fn arvore_com_historico(id: &str, nup: &str) -> String {
    com_link(&arvore_de(id, nup), &[forma_go(&link_do_historico(id))])
}

/// Árvore com o link do histórico na forma da `UnB` (função servida).
pub(super) fn arvore_com_historico_unb(id: &str, nup: &str) -> String {
    com_link(&arvore_de(id, nup), &[forma_unb(&link_do_historico(id))])
}

/// URL absoluta do histórico publicada pela árvore sintética.
pub(super) fn url_historico(id: &str) -> String {
    absoluta(&link_do_historico(id))
}

/// Outro link assinado do mesmo histórico (o guardado numa execução
/// anterior), para distinguir nos pedidos qual foi usado.
pub(super) fn url_historico_guardada(id: &str) -> String {
    url_historico(id).replace(HASH_HISTORICO, &hash('7'))
}

/// `action` absoluto do `form#frmProcedimentoHistorico` sintético.
pub(super) fn url_post_historico(id: &str) -> String {
    absoluta(&action_do_historico(id))
}

/// Página do histórico do NUP padrão, visão `tipo`, legenda `total`.
pub(super) fn historico(tipo: char, total: u64, linhas: usize) -> String {
    pagina_do_historico(NUP, ID, tipo, total, linhas)
}

/// Árvore com o histórico (forma GO) e, na barra do processo, o link
/// `acao` das propriedades.
pub(super) fn arvore_com_propriedades(id: &str, nup: &str, acao: AcaoDoSei) -> String {
    use crate::propriedades_processo::tests_apoio::{
        acao_da_barra, barra, com_barra, link_das_propriedades,
    };
    let acoes = [acao_da_barra(&link_das_propriedades(acao, id))];
    com_barra(&arvore_com_historico(id, nup), 0, &barra(&acoes))
}

/// URL absoluta das propriedades publicada pela barra sintética.
pub(super) fn url_propriedades(acao: AcaoDoSei, id: &str) -> String {
    absoluta(&crate::propriedades_processo::tests_apoio::link_das_propriedades(acao, id))
}

/// Tela de propriedades sintética do NUP e id padrão.
pub(super) fn propriedades() -> String {
    crate::propriedades_processo::tests_apoio::tela_das_propriedades(NUP, ID)
}

/// Uma requisição vista pelo transporte.
#[derive(Debug, PartialEq, Eq)]
pub(super) struct Pedido {
    pub(super) metodo: &'static str,
    pub(super) url: String,
    pub(super) corpo: Option<String>,
}

pub(super) fn get(url: &str) -> Pedido {
    Pedido {
        metodo: "GET",
        url: url.to_owned(),
        corpo: None,
    }
}

pub(super) fn post(url: &str, corpo: &str) -> Pedido {
    Pedido {
        metodo: "POST",
        url: url.to_owned(),
        corpo: Some(corpo.to_owned()),
    }
}

pub(super) const CORPO_RAPIDO: &str = "txtPesquisaRapida=00001.000001%2F2026-01";

/// Resposta roteirizada: status, HTML e `Location`.
pub(super) type Resposta = (i32, String, Option<String>);

pub(super) fn ok(html: String) -> Resposta {
    (200, html, None)
}

pub(super) fn r302(localizacao: &str) -> Resposta {
    (302, String::new(), Some(localizacao.to_owned()))
}

pub(super) struct Roteiro {
    respostas: VecDeque<Resposta>,
    pub(super) pedidos: Vec<Pedido>,
}

impl Roteiro {
    pub(super) fn novo(respostas: Vec<Resposta>) -> Self {
        Self {
            respostas: respostas.into(),
            pedidos: Vec::new(),
        }
    }

    pub(super) fn respostas_restantes(&self) -> usize {
        self.respostas.len()
    }

    fn proxima(&mut self) -> RespostaDoPost<Cursor<Vec<u8>>> {
        let (status_code, html, localizacao) = self
            .respostas
            .pop_front()
            .expect("requisição além do roteiro");
        // Servido em iso-8859-1, como o MIME declara.
        let corpo: Vec<u8> = html
            .chars()
            .map(|c| u8::try_from(c).unwrap_or(b'?'))
            .collect();
        let tamanho = corpo.len();
        RespostaDoPost {
            resposta: crate::transporte::testes_http::resposta(
                status_code,
                "text/html; charset=iso-8859-1",
                corpo,
                Some(tamanho),
            ),
            localizacao,
        }
    }
}

impl EnvioDoControle for Roteiro {
    type Corpo = Cursor<Vec<u8>>;
    fn post(
        &mut self,
        submissao: SubmissaoDerivada,
        cookie: &str,
    ) -> Result<RespostaDoPost<Self::Corpo>, FalhaTransporte> {
        assert_eq!(cookie, COOKIE);
        let corpo = String::from_utf8(submissao.corpo().to_vec()).unwrap();
        self.pedidos.push(post(submissao.url(), &corpo));
        Ok(self.proxima())
    }
    fn get(
        &mut self,
        url: &str,
        cookie: &str,
    ) -> Result<RespostaWeb<Self::Corpo>, FalhaTransporte> {
        assert_eq!(cookie, COOKIE);
        self.pedidos.push(get(url));
        Ok(self.proxima().resposta)
    }
}

impl EnvioDoProcesso for Roteiro {
    fn get_sem_seguir(
        &mut self,
        url: &str,
        cookie: &str,
    ) -> Result<RespostaDoPost<Self::Corpo>, FalhaTransporte> {
        assert_eq!(cookie, COOKIE);
        self.pedidos.push(get(url));
        Ok(self.proxima())
    }

    /// O anexo tem roteiro próprio (`tests_documento_externo`).
    fn get_binario(
        &mut self,
        _: &str,
        _: &str,
        _: usize,
    ) -> Result<crate::transporte::resposta_binaria::RespostaBinaria<Self::Corpo>, FalhaTransporte>
    {
        panic!("GET binário fora do roteiro")
    }
}

/// Diretório temporário com o cookie (0600) e o caminho do índice.
pub(super) struct Ambiente {
    pasta: PathBuf,
    pub(super) cookie: PathBuf,
    pub(super) indice: IndiceLocal,
}

impl Drop for Ambiente {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.pasta);
    }
}

pub(super) fn ambiente() -> Ambiente {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static N: AtomicUsize = AtomicUsize::new(0);
    let pasta = std::env::temp_dir().join(format!(
        "ies-processo-obter-{}-{}",
        std::process::id(),
        N.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&pasta).unwrap();
    crate::indice_local::tests::definir_modo(&pasta, 0o700);
    let cookie = pasta.join("cookie");
    std::fs::write(&cookie, format!("{COOKIE}\n")).unwrap();
    crate::indice_local::tests::definir_modo(&cookie, 0o600);
    let indice = IndiceLocal::em(pasta.join("indice").join("ies.sqlite3"));
    Ambiente {
        pasta,
        cookie,
        indice,
    }
}

pub(super) fn geracao() -> String {
    crate::sessao_login::geracao_da_sessao(COOKIE)
}

pub(super) fn chave<'a>(geracao: &'a str, acao: AcaoDoSei, alvo: &'a str) -> ChaveDoLink<'a> {
    ChaveDoLink {
        geracao_sessao: geracao,
        unidade: ID_UNIDADE,
        acao,
        alvo,
    }
}

/// Roda o fluxo sem andamentos (fatia 1) com `respostas`.
pub(super) fn rodar(
    amb: &Ambiente,
    unidade: Option<&str>,
    respostas: Vec<Resposta>,
) -> (Result<ProcessoAberto, ErroProcessoObter>, Vec<Pedido>) {
    rodar_com(amb, unidade, PedidoDeAndamentos::Omitidos, respostas)
}

/// Roda o fluxo com `respostas`; devolve o resultado e os pedidos feitos.
pub(super) fn rodar_com(
    amb: &Ambiente,
    unidade: Option<&str>,
    andamentos: PedidoDeAndamentos,
    respostas: Vec<Resposta>,
) -> (Result<ProcessoAberto, ErroProcessoObter>, Vec<Pedido>) {
    rodar_nup(amb, NUP, unidade, andamentos, respostas)
}

/// Como [`rodar_com`], pedindo `nup`.
pub(super) fn rodar_nup(
    amb: &Ambiente,
    nup: &str,
    unidade: Option<&str>,
    andamentos: PedidoDeAndamentos,
    respostas: Vec<Resposta>,
) -> (Result<ProcessoAberto, ErroProcessoObter>, Vec<Pedido>) {
    rodar_tudo(amb, nup, unidade, (andamentos, false, false), respostas)
}

/// Como [`rodar_com`], dizendo também se as propriedades são lidas.
pub(super) fn rodar_props(
    amb: &Ambiente,
    andamentos: PedidoDeAndamentos,
    propriedades: bool,
    respostas: Vec<Resposta>,
) -> (Result<ProcessoAberto, ErroProcessoObter>, Vec<Pedido>) {
    rodar_tudo(amb, NUP, None, (andamentos, propriedades, false), respostas)
}

/// Como [`rodar_props`], dizendo também se os marcadores são lidos.
pub(super) fn rodar_marc(
    amb: &Ambiente,
    andamentos: PedidoDeAndamentos,
    (propriedades, marcadores): (bool, bool),
    respostas: Vec<Resposta>,
) -> (Result<ProcessoAberto, ErroProcessoObter>, Vec<Pedido>) {
    rodar_tudo(
        amb,
        NUP,
        None,
        (andamentos, propriedades, marcadores),
        respostas,
    )
}

fn rodar_tudo(
    amb: &Ambiente,
    nup: &str,
    unidade: Option<&str>,
    (andamentos, propriedades, marcadores): (PedidoDeAndamentos, bool, bool),
    respostas: Vec<Resposta>,
) -> (Result<ProcessoAberto, ErroProcessoObter>, Vec<Pedido>) {
    let url = url_controle();
    let sessao = SessaoDoProcesso {
        url_do_controle: &url,
        origem: ORIGEM,
        cookie: &CookieDaSessao::ler(&amb.cookie),
    };
    let nup = NumeroProcesso::novo(nup).unwrap();
    let pedido = PedidoDoProcesso {
        nup: &nup,
        unidade,
        andamentos,
        propriedades,
        marcadores,
        unidade_exigida: false,
        leitura_das_pastas: crate::processo_obter::LeituraDasPastas::Estrita,
    };
    let mut roteiro = Roteiro::novo(respostas);
    let r = obter_processo(&sessao, &pedido, &amb.indice, &mut roteiro);
    assert!(roteiro.respostas.is_empty(), "respostas não consumidas");
    (r, roteiro.pedidos)
}
