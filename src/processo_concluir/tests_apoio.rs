//! Apoio dos testes de `processo concluir`: transporte roteirizado, índice
//! em diretório temporário e telas SINTÉTICAS na estrutura observada
//! (Controle com `tr#P{id}`, tela do processo, árvore com a situação e a
//! barra, tela "Conclusão de Processo" das cinco instalações 4.1/5.0, tela
//! do nó da `UnB` 4.0.9). Hosts, hashes, ids e números inventados.
use super::{ErroConcluir, PedidoDaConclusao, ProcessoConcluido, concluir_processo};
pub(crate) use crate::arvore_processo::tests::NUP;
pub(crate) use crate::arvore_processo::tests_concluir::{
    ACAO_FUNCAO, absoluta, arvore_para_concluir, link_da_funcao, link_da_tela_do_no,
    link_de_concluir, pela_tela, tela_do_no,
};
use crate::arvore_processo::tests_tela::{src_da_arvore, tela_do_processo};
use crate::formulario_web::{RespostaDoPost, SubmissaoDerivada};
use crate::indice_local::IndiceLocal;
use crate::processo::NumeroProcesso;
use crate::processo_obter::{EnvioDoProcesso, SessaoDoProcesso};
use crate::transporte::sessao::CookieDaSessao;
use crate::transporte::{FalhaTransporte, RespostaWeb};
use crate::visao_do_controle::EnvioDoControle;
use std::collections::VecDeque;
use std::fmt::Write as _;
use std::io::Cursor;
use std::path::PathBuf;

pub(crate) const ORIGEM: &str = "https://sei.exemplo.gov.br/sei";
pub(crate) const UNIDADE: &str = "TESTE";
pub(crate) const ID: &str = "10";
const CONTEXTO: &str = "infra_sistema=100000100&infra_unidade_atual=110000001";
const COOKIE: &str = "PHPSESSID=SINTETICO";
pub(crate) const HASH_ACTION: &str =
    "9999999999999999999999999999999999999999999999999999999999999999";

pub(crate) fn url_controle() -> String {
    format!(
        "{ORIGEM}/controlador.php?acao=procedimento_controlar&{CONTEXTO}&infra_hash={}",
        "a".repeat(64)
    )
}

pub(crate) fn trabalhar_controle(id: &str) -> String {
    format!(
        "controlador.php?acao=procedimento_trabalhar&acao_origem=procedimento_controlar&acao_retorno=procedimento_controlar&id_procedimento={id}&{CONTEXTO}&infra_hash={}",
        "d".repeat(64)
    )
}

/// Controle (visão resumida) com o processo `ID` em Recebidos.
pub(crate) fn controle() -> String {
    format!(
        "<html><body><a id=\"lnkInfraUnidade\">{UNIDADE}</a>\
         <table id=\"tblProcessosRecebidos\"><caption>Processos recebidos (1 registros - 1 a 1):</caption>\
         <tr id=\"P{ID}\"><td><a href=\"{}\" class=\"processoVisualizado\">{NUP}</a></td></tr></table>\
         <table id=\"tblProcessosGerados\"><caption>0 registros:</caption></table></body></html>",
        trabalhar_controle(ID)
    )
}

pub(crate) fn trabalhar() -> String {
    tela_do_processo(UNIDADE, &[src_da_arvore(ID)])
}

pub(crate) fn url_arvore() -> String {
    absoluta(&src_da_arvore(ID))
}

pub(crate) fn ancora(sigla: &str) -> String {
    format!(
        "<a alt=\"Unidade {sigla}\" title=\"Unidade {sigla}\" class=\"ancoraSigla\">{sigla}</a>"
    )
}

/// O `.html` da raiz com as unidades abertas (forma `UnB`/GO/PEN).
pub(crate) fn situacao(siglas: &[&str]) -> String {
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
pub(crate) fn arvore(siglas: &[&str], acoes: &[String]) -> String {
    arvore_para_concluir(ID, NUP, &situacao(siglas), acoes)
}

pub(crate) fn action_de_concluir() -> String {
    format!(
        "controlador.php?acao=procedimento_concluir&acao_origem=procedimento_concluir&id_procedimento={ID}&arvore=1&{CONTEXTO}&infra_hash={HASH_ACTION}"
    )
}

/// Tela "Conclusão de Processo" na forma das cinco instalações 4.1/5.0.
pub(crate) fn tela_de_conclusao(id: &str) -> String {
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
pub(crate) const CORPO_DA_CONCLUSAO: &str = "hdnInfraTipoPagina=2&sbmSalvar=Salvar&rdoConcluir=S&txtPrazoReaberturaProgramada=&txtDiasReaberturaProgramada=&hdnIdProtocolos=10&hdnIdProcedimentoDestino=";

/// `Location` do 302 observado depois do POST.
pub(crate) fn volta_a_arvore() -> String {
    format!(
        "controlador.php?acao=arvore_visualizar&acao_origem=procedimento_concluir&atualizar_arvore=1&id_procedimento={ID}&arvore=1&{CONTEXTO}&infra_hash={}",
        "b".repeat(64)
    )
}

/// Página que a forma 4.0 devolve ao GET da função (molde da tela do nó).
pub(crate) fn resposta_da_funcao() -> String {
    tela_do_no(ID).replace("SEI - Visualizar Árvore", "SEI - ")
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Pedido {
    pub(crate) metodo: &'static str,
    pub(crate) url: String,
    pub(crate) corpo: Option<String>,
}

pub(crate) fn get(url: &str) -> Pedido {
    Pedido {
        metodo: "GET",
        url: url.to_owned(),
        corpo: None,
    }
}

pub(crate) fn post(url: &str, corpo: &str) -> Pedido {
    Pedido {
        metodo: "POST",
        url: url.to_owned(),
        corpo: Some(corpo.to_owned()),
    }
}

/// Status, HTML e `Location`; ou falha de transporte.
pub(crate) type Resposta = Result<(i32, String, Option<String>), FalhaTransporte>;

#[expect(
    clippy::unnecessary_wraps,
    reason = "`Resposta` também carrega falhas de transporte"
)]
pub(crate) fn ok(html: String) -> Resposta {
    Ok((200, html, None))
}

#[expect(
    clippy::unnecessary_wraps,
    reason = "`Resposta` também carrega falhas de transporte"
)]
pub(crate) fn r302(localizacao: &str) -> Resposta {
    Ok((302, String::new(), Some(localizacao.to_owned())))
}

pub(crate) struct Roteiro {
    respostas: VecDeque<Resposta>,
    pub(crate) pedidos: Vec<Pedido>,
}

impl Roteiro {
    fn proxima(
        &mut self,
        pedido: Pedido,
    ) -> Result<RespostaDoPost<Cursor<Vec<u8>>>, FalhaTransporte> {
        let (status_code, html, localizacao) = crate::transporte::testes_http::atender(
            &mut self.pedidos,
            &mut self.respostas,
            pedido,
        )?;
        let corpo: Vec<u8> = html
            .chars()
            .map(|c| u8::try_from(c).unwrap_or(b'?'))
            .collect();
        Ok(RespostaDoPost {
            resposta: RespostaWeb {
                status_code,
                // A consulta do autocomplete (processo enviar) responde XML.
                mime: if html.starts_with("<?xml") {
                    "application/xml; charset=iso-8859-1"
                } else {
                    "text/html; charset=iso-8859-1"
                }
                .to_owned(),
                content_length: Some(corpo.len()),
                localizacao: None,
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
        self.proxima(post(submissao.url(), &corpo))
    }
    fn get(
        &mut self,
        url: &str,
        cookie: &str,
    ) -> Result<RespostaWeb<Self::Corpo>, FalhaTransporte> {
        assert_eq!(cookie, COOKIE);
        self.proxima(get(url)).map(|r| r.resposta)
    }
}

impl EnvioDoProcesso for Roteiro {
    fn get_sem_seguir(
        &mut self,
        url: &str,
        cookie: &str,
    ) -> Result<RespostaDoPost<Self::Corpo>, FalhaTransporte> {
        assert_eq!(cookie, COOKIE);
        self.proxima(get(url))
    }
    /// A conclusão não lê anexo.
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

/// Diretório temporário com o cookie (0600) e o índice.
pub(crate) struct Ambiente {
    pasta: PathBuf,
    cookie: PathBuf,
    pub(crate) indice: IndiceLocal,
}

impl Drop for Ambiente {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.pasta);
    }
}

pub(crate) fn ambiente() -> Ambiente {
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

/// Roda `executar` com `respostas` na sessão sintética; confere que todas
/// foram usadas (também a de `processo reabrir`).
pub(crate) fn rodar_com<T>(
    amb: &Ambiente,
    respostas: Vec<Resposta>,
    executar: impl FnOnce(&SessaoDoProcesso<'_>, &NumeroProcesso, &IndiceLocal, &mut Roteiro) -> T,
) -> (T, Vec<Pedido>) {
    let url = url_controle();
    let sessao = SessaoDoProcesso {
        url_do_controle: &url,
        origem: ORIGEM,
        cookie: &CookieDaSessao::ler(&amb.cookie),
    };
    let nup = NumeroProcesso::novo(NUP).unwrap();
    let mut roteiro = Roteiro {
        respostas: respostas.into(),
        pedidos: Vec::new(),
    };
    let r = executar(&sessao, &nup, &amb.indice, &mut roteiro);
    assert!(roteiro.respostas.is_empty(), "respostas não consumidas");
    (r, roteiro.pedidos)
}

/// Roda `processo concluir` com `respostas`; confere que todas foram usadas.
pub(crate) fn rodar(
    amb: &Ambiente,
    confirmar: bool,
    respostas: Vec<Resposta>,
) -> (Result<ProcessoConcluido, ErroConcluir>, Vec<Pedido>) {
    rodar_com(amb, respostas, |sessao, nup, indice, roteiro| {
        let pedido = PedidoDaConclusao {
            nup,
            unidade: None,
            confirmar,
        };
        concluir_processo(sessao, &pedido, indice, roteiro)
    })
}

/// Abertura a frio pela linha do Controle: as três respostas e os pedidos.
pub(crate) fn abertura(arvore: String) -> Vec<Resposta> {
    vec![ok(controle()), ok(trabalhar()), ok(arvore)]
}

pub(crate) fn pedidos_da_abertura() -> Vec<Pedido> {
    vec![
        get(&url_controle()),
        get(&absoluta(&trabalhar_controle(ID))),
        get(&url_arvore()),
    ]
}

pub(crate) fn vazio() -> &'static [String] {
    &[]
}

pub(crate) fn funcao() -> Vec<String> {
    vec![ACAO_FUNCAO.to_owned()]
}

pub(crate) fn tela() -> Vec<String> {
    vec![pela_tela(ID)]
}

pub(crate) fn links() -> (String, String, String) {
    (
        absoluta(&link_de_concluir(ID)),
        absoluta(&link_da_tela_do_no(ID)),
        absoluta(&link_da_funcao(ID)),
    )
}

/// Cobertura da contagem real (issue #184), inclusive recusas e falha de rede.
/// A bancada registra o pedido antes da resposta; parsing e índice não entram.
#[cfg(test)]
mod tests_contagem {
    use super::*;
    use crate::processo_concluir::escrita::Contador;

    #[test]
    fn get_200_conta_uma_tentativa() {
        let mut roteiro = Roteiro {
            respostas: vec![ok("<html></html>".to_owned())].into(),
            pedidos: Vec::new(),
        };
        let mut rede = Contador::novo(&mut roteiro);
        let resposta = rede.get_sem_seguir(ORIGEM, COOKIE).unwrap();
        assert_eq!(resposta.resposta.status_code, 200);
        assert_eq!(rede.feitas(), 1);
        assert_eq!(roteiro.pedidos, vec![get(ORIGEM)]);
    }

    #[test]
    fn redirecionamento_302_e_dado_e_nao_dispara_outra_requisicao() {
        let mut roteiro = Roteiro {
            respostas: vec![r302("controlador.php?acao=teste")].into(),
            pedidos: Vec::new(),
        };
        let mut rede = Contador::novo(&mut roteiro);
        let resposta = rede.get_sem_seguir(ORIGEM, COOKIE).unwrap();
        assert_eq!(resposta.resposta.status_code, 302);
        assert_eq!(
            resposta.localizacao.as_deref(),
            Some("controlador.php?acao=teste")
        );
        assert_eq!(rede.feitas(), 1);
        assert_eq!(roteiro.pedidos, vec![get(ORIGEM)]);
    }

    #[test]
    fn falha_na_segunda_tentativa_tambem_conta_sem_retry() {
        let mut roteiro = Roteiro {
            respostas: vec![ok("<html></html>".to_owned()), Err(FalhaTransporte::Rede)].into(),
            pedidos: Vec::new(),
        };
        let mut rede = Contador::novo(&mut roteiro);
        assert!(rede.get_sem_seguir(ORIGEM, COOKIE).is_ok());
        assert!(matches!(
            rede.get_sem_seguir(ORIGEM, COOKIE),
            Err(FalhaTransporte::Rede)
        ));
        assert_eq!(rede.feitas(), 2);
        assert_eq!(roteiro.pedidos, vec![get(ORIGEM), get(ORIGEM)]);
    }

    #[test]
    fn post_recusado_tambem_conta_no_sucesso_reconciliado() {
        let amb = ambiente();
        let mut respostas = abertura(arvore(&[UNIDADE], &tela()));
        respostas.extend([
            ok(tela_de_conclusao(ID)),
            Ok((503, String::new(), None)),
            ok(arvore(&[], vazio())),
        ]);
        let (resultado, pedidos) = rodar(&amb, true, respostas);
        let feito = resultado.unwrap();
        assert_eq!(feito.requisicoes, 6);
        assert_eq!(pedidos.len(), 6);
        assert_eq!(pedidos.iter().filter(|p| p.metodo == "POST").count(), 1);
    }
}
