//! Apoio dos testes de `processo concluir`: transporte roteirizado, índice
//! em diretório temporário e telas SINTÉTICAS na estrutura observada
//! (Controle com `tr#P{id}`, tela do processo, árvore com a situação e a
//! barra, tela "Conclusão de Processo" das cinco instalações 4.1/5.0, tela
//! do nó da `UnB` 4.0.9). Hosts, hashes, ids e números inventados.
use super::{ErroConcluir, PedidoDaConclusao, ProcessoConcluido, concluir_processo};
pub(super) use crate::arvore_processo::tests::NUP;
pub(super) use crate::arvore_processo::tests_concluir::{
    ACAO_FUNCAO, absoluta, arvore_para_concluir, link_da_funcao, link_da_tela_do_no,
    link_de_concluir, pela_tela, tela_do_no,
};
use crate::arvore_processo::tests_tela::{src_da_arvore, tela_do_processo};
use crate::formulario_web::{RespostaDoPost, SubmissaoDerivada};
use crate::indice_local::IndiceLocal;
use crate::processo::NumeroProcesso;
use crate::processo_obter::{EnvioDoProcesso, SessaoDoProcesso};
use crate::transporte::{FalhaTransporte, RespostaWeb};
use crate::visao_do_controle::EnvioDoControle;
use std::collections::VecDeque;
use std::fmt::Write as _;
use std::io::Cursor;
use std::path::PathBuf;

pub(super) const ORIGEM: &str = "https://sei.exemplo.gov.br/sei";
pub(super) const UNIDADE: &str = "TESTE";
pub(super) const ID: &str = "10";
const CONTEXTO: &str = "infra_sistema=100000100&infra_unidade_atual=110000001";
const COOKIE: &str = "PHPSESSID=SINTETICO";
pub(super) const HASH_ACTION: &str =
    "9999999999999999999999999999999999999999999999999999999999999999";

pub(super) fn url_controle() -> String {
    format!(
        "{ORIGEM}/controlador.php?acao=procedimento_controlar&{CONTEXTO}&infra_hash={}",
        "a".repeat(64)
    )
}

pub(super) fn trabalhar_controle(id: &str) -> String {
    format!(
        "controlador.php?acao=procedimento_trabalhar&acao_origem=procedimento_controlar&acao_retorno=procedimento_controlar&id_procedimento={id}&{CONTEXTO}&infra_hash={}",
        "d".repeat(64)
    )
}

/// Controle (visão resumida) com o processo `ID` em Recebidos.
pub(super) fn controle() -> String {
    format!(
        "<html><body><a id=\"lnkInfraUnidade\">{UNIDADE}</a>\
         <table id=\"tblProcessosRecebidos\"><caption>1 registros - 1 a 1):</caption>\
         <tr id=\"P{ID}\"><td><a href=\"{}\" class=\"processoVisualizado\">{NUP}</a></td></tr></table>\
         <table id=\"tblProcessosGerados\"><caption>0 registros:</caption></table></body></html>",
        trabalhar_controle(ID)
    )
}

pub(super) fn trabalhar() -> String {
    tela_do_processo(UNIDADE, &[src_da_arvore(ID)])
}

pub(super) fn url_arvore() -> String {
    absoluta(&src_da_arvore(ID))
}

pub(super) fn ancora(sigla: &str) -> String {
    format!(
        "<a alt=\"Unidade {sigla}\" title=\"Unidade {sigla}\" class=\"ancoraSigla\">{sigla}</a>"
    )
}

/// O `.html` da raiz com as unidades abertas (forma `UnB`/GO/PEN).
pub(super) fn situacao(siglas: &[&str]) -> String {
    match siglas {
        [] => "Processo não possui andamentos abertos.<br />".to_owned(),
        [uma] => format!("Processo aberto somente na unidade {}.<br />", ancora(uma)),
        varias => {
            let mut texto = String::from("Processo aberto nas unidades:<br />");
            for sigla in varias {
                let _ = write!(texto, "{}<br />", ancora(sigla));
            }
            texto.push_str("<br />");
            texto
        }
    }
}

/// Árvore com a situação `siglas` e a barra com `acoes`.
pub(super) fn arvore(siglas: &[&str], acoes: &[String]) -> String {
    arvore_para_concluir(ID, NUP, &situacao(siglas), acoes)
}

pub(super) fn action_de_concluir() -> String {
    format!(
        "controlador.php?acao=procedimento_concluir&acao_origem=procedimento_concluir&id_procedimento={ID}&arvore=1&{CONTEXTO}&infra_hash={HASH_ACTION}"
    )
}

/// Tela "Conclusão de Processo" na forma das cinco instalações 4.1/5.0.
pub(super) fn tela_de_conclusao(id: &str) -> String {
    format!(
        "<html><head><title>SEI - Concluir Processo</title><script>function validarConcluir(){{ return true; }}</script></head><body>\
         <div id=\"divInfraBarraLocalizacao\" class=\"infraBarraLocalizacao\">Conclusão de Processo</div>\
         <form id=\"frmDesentranharDocumento\" method=\"post\" onsubmit=\"return OnSubmitForm();\" action=\"{action}\">\n\
         <input type=\"hidden\" id=\"hdnInfraTipoPagina\" name=\"hdnInfraTipoPagina\" value=\"2\"/>\n\
         <div id=\"divInfraBarraComandosSuperior\" class=\"infraBarraComandos\">\
         <button tabindex=\"451\" type=\"submit\" accesskey=\"S\" name=\"sbmSalvar\" id=\"sbmSalvar\" value=\"Salvar\" class=\"infraButton\"><span class=\"infraTeclaAtalho\">S</span>alvar</button>\
         <button tabindex=\"452\" type=\"button\" accesskey=\"C\" id=\"btnCancelar\" name=\"btnCancelar\" value=\"Cancelar\" onclick=\"location.href='controlador.php?acao=arvore_visualizar';\" class=\"infraButton\">Cancelar</button></div>\n\
         <select id=\"selProcedimentos\" name=\"selProcedimentos\" size=\"4\" class=\"infraSelect\" multiple=\"multiple\" tabindex=\"501\">\n\
         <option value=\"{id}\">{NUP} - Tipo Sintetico</option>\n</select>\n\
         <input type=\"radio\" name=\"rdoConcluir\" id=\"optSomenteConcluir\" onclick=\"mostrarDataProgramada();\" value=\"S\" checked=\"checked\" class=\"infraRadio\" tabindex=\"502\" />\n\
         <input type=\"radio\" name=\"rdoConcluir\" id=\"optConcluirAgendar\" onclick=\"mostrarDataProgramada();\" value=\"V\"  class=\"infraRadio\" tabindex=\"503\" />\n\
         <input type=\"radio\" name=\"rdoPrazoReaberturaProgramada\" id=\"optDataCertaReaberturaProgramada\" onclick=\"configurarReaberturaProgramada();\"  value=\"1\" class=\"infraRadio\"/>\n\
         <input type=\"text\" id=\"txtPrazoReaberturaProgramada\" name=\"txtPrazoReaberturaProgramada\" onkeypress=\"return infraMascaraData(this, event)\" class=\"infraText\" value=\"\" tabindex=\"505\" />\n\
         <input type=\"radio\" name=\"rdoPrazoReaberturaProgramada\" id=\"optDiasReaberturaProgramada\" onclick=\"configurarReaberturaProgramada();\"  value=\"2\" class=\"infraRadio\"/>\n\
         <input type=\"text\" id=\"txtDiasReaberturaProgramada\" name=\"txtDiasReaberturaProgramada\" class=\"infraText\" value=\"\" onkeypress=\"return infraMascaraNumero(this,event);\" maxlength=\"3\" tabindex=\"508\" />\n\
         <input type=\"checkbox\" id=\"chkSinDiasUteisReaberturaProgramada\" name=\"chkSinDiasUteisReaberturaProgramada\" class=\"infraCheckbox\"  tabindex=\"509\" />\n\
         <input type=\"hidden\" id=\"hdnIdProtocolos\" name=\"hdnIdProtocolos\" value=\"{id}\" />\n\
         <input type=\"hidden\" id=\"hdnIdProcedimentoDestino\" name=\"hdnIdProcedimentoDestino\" value=\"\" />\n\
         </form>\n<input type=\"hidden\" id=\"hdnInfraPrefixoCookie\" name=\"hdnInfraPrefixoCookie\" value=\"SEI_X\"/>\n</body></html>",
        action = action_de_concluir()
    )
}

/// O corpo que o navegador envia (ordem do documento).
pub(super) const CORPO_DA_CONCLUSAO: &str = "hdnInfraTipoPagina=2&sbmSalvar=Salvar&rdoConcluir=S&txtPrazoReaberturaProgramada=&txtDiasReaberturaProgramada=&hdnIdProtocolos=10&hdnIdProcedimentoDestino=";

/// `Location` do 302 observado depois do POST.
pub(super) fn volta_a_arvore() -> String {
    format!(
        "controlador.php?acao=arvore_visualizar&acao_origem=procedimento_concluir&atualizar_arvore=1&id_procedimento={ID}&arvore=1&{CONTEXTO}&infra_hash={}",
        "b".repeat(64)
    )
}

/// Página que a forma 4.0 devolve ao GET da função (molde da tela do nó).
pub(super) fn resposta_da_funcao() -> String {
    tela_do_no(ID).replace("SEI - Visualizar Árvore", "SEI - ")
}

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

/// Status, HTML e `Location`; ou falha de transporte.
pub(super) type Resposta = Result<(i32, String, Option<String>), FalhaTransporte>;

#[allow(clippy::unnecessary_wraps)] // `Resposta` também carrega falhas de transporte.
pub(super) fn ok(html: String) -> Resposta {
    Ok((200, html, None))
}

#[allow(clippy::unnecessary_wraps)]
pub(super) fn r302(localizacao: &str) -> Resposta {
    Ok((302, String::new(), Some(localizacao.to_owned())))
}

pub(super) struct Roteiro {
    respostas: VecDeque<Resposta>,
    pub(super) pedidos: Vec<Pedido>,
}

impl Roteiro {
    fn proxima(&mut self) -> Result<RespostaDoPost<Cursor<Vec<u8>>>, FalhaTransporte> {
        let (status_code, html, localizacao) = self
            .respostas
            .pop_front()
            .expect("requisição além do roteiro")?;
        let corpo: Vec<u8> = html
            .chars()
            .map(|c| u8::try_from(c).unwrap_or(b'?'))
            .collect();
        Ok(RespostaDoPost {
            resposta: RespostaWeb {
                status_code,
                mime: "text/html; charset=iso-8859-1".to_owned(),
                content_length: Some(corpo.len()),
                corpo: Cursor::new(corpo),
            },
            localizacao,
        })
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
        self.proxima()
    }
    fn get(
        &mut self,
        url: &str,
        cookie: &str,
    ) -> Result<RespostaWeb<Self::Corpo>, FalhaTransporte> {
        assert_eq!(cookie, COOKIE);
        self.pedidos.push(get(url));
        self.proxima().map(|r| r.resposta)
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
        self.proxima()
    }
}

/// Diretório temporário com o cookie (0600) e o índice.
pub(super) struct Ambiente {
    pasta: PathBuf,
    cookie: PathBuf,
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
        "ies-processo-concluir-{}-{}",
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

/// Roda `processo concluir` com `respostas`; confere que todas foram usadas.
pub(super) fn rodar(
    amb: &Ambiente,
    confirmar: bool,
    respostas: Vec<Resposta>,
) -> (Result<ProcessoConcluido, ErroConcluir>, Vec<Pedido>) {
    let url = url_controle();
    let sessao = SessaoDoProcesso {
        url_do_controle: &url,
        origem: ORIGEM,
        cookie_file: &amb.cookie,
    };
    let nup = NumeroProcesso::novo(NUP).unwrap();
    let pedido = PedidoDaConclusao {
        nup: &nup,
        unidade: None,
        confirmar,
    };
    let mut roteiro = Roteiro {
        respostas: respostas.into(),
        pedidos: Vec::new(),
    };
    let r = concluir_processo(&sessao, &pedido, &amb.indice, &mut roteiro);
    assert!(roteiro.respostas.is_empty(), "respostas não consumidas");
    (r, roteiro.pedidos)
}

/// Abertura a frio pela linha do Controle: as três respostas e os pedidos.
pub(super) fn abertura(arvore: String) -> Vec<Resposta> {
    vec![ok(controle()), ok(trabalhar()), ok(arvore)]
}

pub(super) fn pedidos_da_abertura() -> Vec<Pedido> {
    vec![
        get(&url_controle()),
        get(&absoluta(&trabalhar_controle(ID))),
        get(&url_arvore()),
    ]
}

pub(super) fn vazio() -> &'static [String] {
    &[]
}

pub(super) fn funcao() -> Vec<String> {
    vec![ACAO_FUNCAO.to_owned()]
}

pub(super) fn tela() -> Vec<String> {
    vec![pela_tela(ID)]
}

#[allow(dead_code)]
pub(super) fn links() -> (String, String, String) {
    (
        absoluta(&link_de_concluir(ID)),
        absoluta(&link_da_tela_do_no(ID)),
        absoluta(&link_da_funcao(ID)),
    )
}
