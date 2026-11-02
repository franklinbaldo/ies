//! Telas sintéticas modeladas na **estrutura** das telas públicas de login
//! (`UnB`, PEN1, UFRJ, GO, 02/11/2026), sem cópia de conteúdo real, e um
//! transporte de roteiro que registra cada pedido. Sem rede.
use super::{PedidoDeLogin, RespostaHttp, TransporteDeLogin};
use crate::configuracao::ArquivosDeAcesso;
use crate::formulario_web::SubmissaoDerivada;
use crate::transporte::FalhaTransporte;
use std::collections::VecDeque;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::time::Duration;

pub(super) const HOST: &str = "sei.exemplo.gov.br";
pub(super) const RAIZ: &str = "https://sei.exemplo.gov.br/";
pub(super) const ORIGEM: &str = "https://sei.exemplo.gov.br/sei";
pub(super) const LOGIN_SIP: &str =
    "https://sei.exemplo.gov.br/sip/login.php?sigla_orgao_sistema=ORGAO-A&sigla_sistema=SEI";
pub(super) const LOGIN_SIP_INFRA_URL: &str = "https://sei.exemplo.gov.br/sip/login.php?sigla_orgao_sistema=ORGAO-A&sigla_sistema=SEI&infra_url=L3NlaS8=";
pub(super) const MODULO_GO: &str = "sei.exemplo.gov.br/sei/modulos/goias/login-sei/MdGoiasLoginSEI.php?acao=md_goias_login_SEI&sigla_orgao_sistema=GOVERNADORIA&sigla_sistema=SEI&infra_url=L3NlaS8=";
pub(super) const HASH: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

pub(super) fn url_do_controle() -> String {
    format!(
        "https://{HOST}/sei/controlador.php?acao=procedimento_controlar&acao_origem=principal&infra_sistema=100000100&infra_unidade_atual=110000849&infra_hash={HASH}"
    )
}

/// Credenciais sintéticas: a senha tem `é` (ISO-8859-1) e `=`.
pub(super) const CREDENCIAIS: &str = "usuario=usuario.teste\nsenha=s\u{e9}nha=1\norgao=ORGAO-A\n";
pub(super) const CORPO_SENHA: &str = "txtUsuario=usuario.teste&pwdSenha=s%E9nha%3D1";

const SCRIPT_ACESSAR: &str = "<script type=\"text/javascript\">
function acaoLogin(acao){

  document.getElementById('hdnAcao').value = acao;

  if (acao == 2 || acao == 3){
    return validarCampos();
  }
  document.getElementById('frmLogin').submit();
  return true;
}
</script>";

const SCRIPT_SBM: &str = "<script type=\"text/javascript\">
var infraCaptchaHttpRequest;
function infraGerarNovoCaptcha(url) { infraCaptchaHttpRequest = new XMLHttpRequest(); }
  function acaoLogin(acao,identificacaoSso = null) {

    document.getElementById('hdnAcao').value = acao;

    if (acao == 2 || acao == 3) {
      if (validarCampos(true)){

        if (document.getElementById('sbmAcessar')!=null) {
          document.getElementById('sbmAcessar').disabled = true;
        }

        return true;
      }
      return false;
    }
    document.getElementById('frmLogin').submit();
    return true;
  }
</script>";

const CSS_DO_CAPTCHA: &str =
    "<style>#divInfraCaptcha {margin:.3em 0;} #txtInfraCaptcha {max-width:100px;}</style>";

const AVISO_DO_NAVEGADOR: &str = "<div id=\"divInfraNavegadorAviso\" class=\"alert alert-danger\" role=\"alert\"><span>O seu navegador precisa ser atualizado.</span></div>";

fn opcoes(lista: &[(&str, &str)]) -> String {
    let mut html = String::from("<option value=\"null\" selected=\"selected\">&nbsp;</option>\n");
    for (valor, texto) in lista {
        let _ = writeln!(html, "<option value=\"{valor}\">{texto}</option>");
    }
    html
}

/// Formulário com `action`, opções, botão e trecho extra dentro do `form`.
pub(super) fn formulario(action: &str, orgaos: &str, botao: &str, extra: &str) -> String {
    format!(
        "<form id=\"frmLogin\" name=\"frmLogin\" class=\"h-100\" action=\"{action}\" method=\"post\" onsubmit=\"return acaoLogin(2);\">
<div id=\"divUsuario\"><input type=\"text\" autofocus=\"\" id=\"txtUsuario\" name=\"txtUsuario\" placeholder=\"Usuário\" class=\"form-control\" value=\"\" tabindex=\"501\" maxlength=\"100\" ></div>
<div id=\"divSenha\"><input type=\"password\" autofocus=\"\" id=\"pwdSenha\" name=\"pwdSenha\" class=\"form-control\" value=\"\" autocomplete=\"off\"></div>
<div id=\"divOrgao\"><select id=\"selOrgao\" name=\"selOrgao\" class=\"form-select\" tabindex=\"503\">
{orgaos}</select></div>
<div class=\"md-form d-flex media\">
<button type=\"submit\" id=\"{botao}\" name=\"{botao}\" class=\"btn text-white\" accesskey=\"e\" tabindex=\"505\">
  ACESSAR
</button></div>
{extra}
<input type=\"hidden\" id=\"hdnAcao\" name=\"hdnAcao\" value=\"1\"/>
</form>"
    )
}

fn pagina(cabeca: &str, corpo: &str) -> String {
    format!(
        "<!DOCTYPE html><html lang=\"pt-br\"><head><meta charset=\"iso-8859-1\">{cabeca}</head><body>{corpo}
<input type=\"hidden\" id=\"hdnInfraPrefixoCookie\" name=\"hdnInfraPrefixoCookie\" value=\"Sistema\"/></body></html>"
    )
}

/// Família `UnB`: `Acessar`, órgão já selecionado, sem `infra_url`.
pub(super) fn tela_acessar() -> String {
    let orgaos = "<option value=\"null\">&nbsp;</option>\n<option value=\"0\" selected=\"selected\">ORGAO-A</option>\n";
    pagina(
        SCRIPT_ACESSAR,
        &formulario(
            "login.php?sigla_orgao_sistema=ORGAO-A&sigla_sistema=SEI",
            orgaos,
            "Acessar",
            "",
        ),
    )
}

/// Família PEN1/UFRJ: `sbmAcessar` desabilitado pelo `acaoLogin(2)`,
/// captcha só em CSS e JavaScript, aviso de navegador.
pub(super) fn tela_sbm() -> String {
    let extra = "<div class=\"text-right mt-1\"><a href=\"#\" class=\"pl-1 linkLogin\" onclick=\"acaoLogin(5)\" tabindex=\"506\">Autenticação em dois fatores</a></div>";
    pagina(
        &format!("{CSS_DO_CAPTCHA}{SCRIPT_SBM}"),
        &format!(
            "{AVISO_DO_NAVEGADOR}{}",
            formulario(
                "login.php?sigla_orgao_sistema=ORGAO-A&sigla_sistema=SEI&infra_url=L3NlaS8=",
                &opcoes(&[("0", "ORGAO-A"), ("1", "TESTE")]),
                "sbmAcessar",
                extra,
            )
        ),
    )
}

/// Família GO: módulo próprio, `action` relativo ao módulo, muitos órgãos.
pub(super) fn tela_go() -> String {
    pagina(
        &format!("{CSS_DO_CAPTCHA}{SCRIPT_SBM}"),
        &formulario(
            "login.php?sigla_orgao_sistema=GOVERNADORIA&sigla_sistema=SEI&infra_url=L3NlaS8=",
            &opcoes(&[
                ("56", "CELG D"),
                ("10269", "CELG G T"),
                ("60", "GOIASG\u{c1}S"),
                ("1", "GOVERNADORIA"),
                ("5", "SEAD"),
            ]),
            "sbmAcessar",
            "",
        ),
    )
}

/// Tela de login devolvida com a recusa declarada e o aviso de navegador.
pub(super) fn tela_recusada(mensagem: &str) -> String {
    let aviso = if mensagem.is_empty() {
        String::new()
    } else {
        format!("<div class=\"alert alert-danger\" role=\"alert\">{mensagem}</div>")
    };
    tela_sbm().replace("<form id=", &format!("{aviso}<form id="))
}

pub(super) fn controle(unidade: &str) -> String {
    format!(
        "<html><body><a id=\"lnkInfraUnidade\" href=\"#\">{unidade}</a>
<table id=\"tblProcessosRecebidos\"><caption>0 registros:</caption></table>
<table id=\"tblProcessosGerados\"><caption>0 registros:</caption></table></body></html>"
    )
}

/// Painel de Controle: o menu publica o link assinado do Controle.
pub(super) fn painel(onclick: &str) -> String {
    format!(
        "<html><body><a id=\"lnkInfraUnidade\" href=\"#\">UNIDADE-TESTE</a>
<a class=\"align-self-center d-none d-md-block\" id=\"lnkControleProcessos\" href=\"#\" onclick=\"{onclick}\" title=\"Controle de Processos\" tabindex=\"55\">Controle</a>
<div id=\"divPainelControle\">Painel</div></body></html>"
    )
}

pub(super) fn onclick_do_controle() -> String {
    format!(
        "location.href='controlador.php?acao=procedimento_controlar&amp;reset=1&amp;infra_sistema=100000100&amp;infra_unidade_atual=110000849&amp;infra_hash={HASH}'"
    )
}

pub(super) fn latin1(texto: &str) -> Vec<u8> {
    texto
        .chars()
        .map(|c| u8::try_from(u32::from(c)).expect("fixture em ISO-8859-1"))
        .collect()
}

fn com_cookies(mut cabecalhos: Vec<(String, String)>, cookies: &[&str]) -> Vec<(String, String)> {
    for cookie in cookies {
        cabecalhos.push(("Set-Cookie".to_owned(), (*cookie).to_owned()));
    }
    cabecalhos
}

pub(super) fn r302(localizacao: &str, cookies: &[&str]) -> RespostaHttp {
    status(302, Some(localizacao), cookies, "", "text/html")
}

pub(super) fn r200(html: &str, cookies: &[&str]) -> RespostaHttp {
    status(200, None, cookies, html, "text/html; charset=iso-8859-1")
}

pub(super) fn status(
    codigo: u16,
    localizacao: Option<&str>,
    cookies: &[&str],
    corpo: &str,
    mime: &str,
) -> RespostaHttp {
    let mut cabecalhos = vec![("Content-Type".to_owned(), mime.to_owned())];
    if let Some(local) = localizacao {
        cabecalhos.push(("Location".to_owned(), local.to_owned()));
    }
    RespostaHttp {
        status: codigo,
        cabecalhos: com_cookies(cabecalhos, cookies),
        corpo: latin1(corpo),
    }
}

/// Um pedido como o transporte o recebeu.
#[derive(Debug, Clone)]
pub(super) struct Pedido {
    pub(super) metodo: &'static str,
    pub(super) url: String,
    pub(super) cookie: String,
    pub(super) nomes: Vec<String>,
    pub(super) corpo: String,
    pub(super) origem: String,
    pub(super) referer: String,
}

/// Responde na ordem do roteiro; pedido além dele falha o teste.
pub(super) struct Roteiro {
    respostas: VecDeque<Result<RespostaHttp, FalhaTransporte>>,
    pub(super) pedidos: Vec<Pedido>,
}

impl Roteiro {
    pub(super) fn novo(respostas: Vec<RespostaHttp>) -> Self {
        Self::com_falhas(respostas.into_iter().map(Ok).collect())
    }
    pub(super) fn com_falhas(respostas: Vec<Result<RespostaHttp, FalhaTransporte>>) -> Self {
        Self {
            respostas: respostas.into(),
            pedidos: Vec::new(),
        }
    }
    fn proxima(&mut self) -> Result<RespostaHttp, FalhaTransporte> {
        self.respostas
            .pop_front()
            .unwrap_or_else(|| panic!("pedido além do roteiro: {:?}", self.pedidos.last()))
    }
    pub(super) fn sequencia(&self) -> Vec<String> {
        self.pedidos
            .iter()
            .map(|p| format!("{} {}", p.metodo, p.url))
            .collect()
    }
    pub(super) fn esgotado(&self) -> bool {
        self.respostas.is_empty()
    }
}

impl TransporteDeLogin for Roteiro {
    fn get(&mut self, url: &str, cookie: &str) -> Result<RespostaHttp, FalhaTransporte> {
        self.pedidos.push(Pedido {
            metodo: "GET",
            url: url.to_owned(),
            cookie: cookie.to_owned(),
            nomes: Vec::new(),
            corpo: String::new(),
            origem: String::new(),
            referer: String::new(),
        });
        self.proxima()
    }

    fn post(
        &mut self,
        submissao: SubmissaoDerivada,
        cookie: &str,
    ) -> Result<RespostaHttp, FalhaTransporte> {
        self.pedidos.push(Pedido {
            metodo: "POST",
            url: submissao.url().to_owned(),
            cookie: cookie.to_owned(),
            nomes: submissao.nomes_dos_campos().to_vec(),
            corpo: String::from_utf8(submissao.corpo().to_vec()).expect("corpo ASCII"),
            origem: submissao.origem(),
            referer: submissao.referer().to_owned(),
        });
        self.proxima()
    }
}

/// Diretório privado com origem e credenciais; sem sessão gravada.
pub(super) struct Ambiente {
    pub(super) dir: PathBuf,
    pub(super) arquivos: ArquivosDeAcesso,
    pub(super) credenciais: PathBuf,
}

pub(super) fn privado(caminho: &Path, conteudo: &str) {
    use std::os::unix::fs::PermissionsExt;
    std::fs::write(caminho, conteudo).unwrap();
    std::fs::set_permissions(caminho, std::fs::Permissions::from_mode(0o600)).unwrap();
}

impl Ambiente {
    pub(super) fn novo() -> Self {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static SEQUENCIA: AtomicUsize = AtomicUsize::new(0);
        let dir = std::env::temp_dir().join(format!(
            "ies-login-{}-{}",
            std::process::id(),
            SEQUENCIA.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let arquivos = ArquivosDeAcesso {
            url_controle: dir.join("url-controle"),
            cookie: dir.join("cookie"),
            origem: dir.join("origem"),
        };
        privado(&arquivos.origem, &format!("{ORIGEM}\n"));
        let credenciais = dir.join("credenciais");
        privado(&credenciais, CREDENCIAIS);
        Self {
            dir,
            arquivos,
            credenciais,
        }
    }

    pub(super) fn pedido(&self, forcar: bool) -> PedidoDeLogin<'_> {
        PedidoDeLogin {
            arquivos: &self.arquivos,
            credenciais: Some(&self.credenciais),
            forcar,
            unidade_esperada: None,
            espera_do_lock: Duration::from_millis(200),
        }
    }

    pub(super) fn gravar_sessao(&self, url: &str, cookie: &str) {
        privado(&self.arquivos.url_controle, &format!("{url}\n"));
        privado(&self.arquivos.cookie, &format!("{cookie}\n"));
    }

    pub(super) fn ler(caminho: &Path) -> String {
        std::fs::read_to_string(caminho).unwrap()
    }

    pub(super) fn geracao(&self) -> PathBuf {
        super::gravacao::caminho_ao_lado(&self.arquivos.url_controle, ".geracao")
    }

    pub(super) fn modo(caminho: &Path) -> u32 {
        use std::os::unix::fs::PermissionsExt;
        std::fs::metadata(caminho).unwrap().permissions().mode() & 0o777
    }

    /// Nomes de arquivo do diretório, em ordem.
    pub(super) fn nomes(&self) -> Vec<String> {
        let mut nomes: Vec<String> = std::fs::read_dir(&self.dir)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        nomes.sort();
        nomes
    }
}

impl Drop for Ambiente {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}
