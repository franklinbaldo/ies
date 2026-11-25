//! Telas do login, lidas na árvore html5ever (sem regex): o
//! formulário `frmLogin` servido, captcha e segundo fator exigidos, a
//! mensagem de recusa e o link do Controle no Painel. As `const` da
//! operação ficam aqui; o primitivo de formulário deriva o POST.
use super::{Credenciais, ErroLogin};
use crate::acao_do_sei::AcaoDoSei;
use crate::formulario_web::{
    ConferenciaDaUnidade, EfeitoAvaliado, EnvioDoFormulario, EstadoDeSessao, OperacaoDeFormulario,
    ResultadoDoEnvio, SubmissaoDerivada, TelaServida, derivar_submissao_de_login,
    localizar_formulario,
};
use crate::instalacao::Instalacao;

const CAMPOS_DO_LOGIN: &[&str] = &["txtUsuario", "pwdSenha", "selOrgao"];

/// `hdnAcao` é gravado pelo `acaoLogin(N)` do `onsubmit` servido.
const OCULTOS_REESCRITOS: &[&str] = &["hdnAcao"];

/// Login com o botão `Acessar` (`UnB` 4.0.9): o `acaoLogin(2)` servido não o
/// desabilita, e ele participa do corpo com o valor servido.
pub(crate) const ENTRAR_COM_ACESSAR: OperacaoDeFormulario = OperacaoDeFormulario {
    acao: AcaoDoSei::Login,
    formulario: "frmLogin",
    envio: EnvioDoFormulario::BotaoDeEnvio("Acessar"),
    campos_ocultos_exigidos: OCULTOS_REESCRITOS,
    campos_ocultos_reescritos: OCULTOS_REESCRITOS,
    campos_usuario: CAMPOS_DO_LOGIN,
    resultado: ResultadoDoEnvio::ConferidoPorLeituraPosterior,
    // A tela de login não publica unidade; o Controle obtido é conferido.
    unidade: ConferenciaDaUnidade::NaTelaDeOrigemDoLink,
    efeito: EfeitoAvaliado::AlteraEstadoDaSessao {
        o_que: EstadoDeSessao::SessaoAutenticada,
    },
};

/// Login com `sbmAcessar` (PEN1, UFRJ, GO): o `acaoLogin(2)` servido o
/// desabilita antes de o navegador montar o corpo; ele não participa.
pub(crate) const ENTRAR_COM_SBM_ACESSAR: OperacaoDeFormulario = OperacaoDeFormulario {
    envio: EnvioDoFormulario::BotaoDesabilitadoPeloOnsubmit("sbmAcessar"),
    ..ENTRAR_COM_ACESSAR
};

/// Trecho literal do `acaoLogin` servido nas quatro instalações.
pub(crate) const HDN_ACAO_PELO_ARGUMENTO: &str = "document.getElementById('hdnAcao').value = acao;";

/// Trecho literal que desabilita `sbmAcessar` (PEN1, UFRJ, GO).
pub(crate) const SBM_ACESSAR_DESABILITADO: &str =
    "document.getElementById('sbmAcessar').disabled = true;";

/// Contêineres que o SEI usa para recusar (Todos, `mensagens.py`); o aviso
/// de navegador usa `alert-danger`, mas é banner de layout.
const CLASSES_DE_RECUSA: [&str; 3] = ["alert-danger", "infraMensagemErro", "infraExcecao"];
const AVISO_DO_NAVEGADOR: &str = "divInfraNavegadorAviso";

/// Os elementos do documento, em ordem.
fn tags(dom: &crate::tela_html::Documento) -> impl Iterator<Item = crate::tela_html::Elemento<'_>> {
    dom.nos()
        .filter_map(|no| match no {
            crate::tela_html::No::Elemento(elemento) => Some(elemento),
            crate::tela_html::No::Texto(_) | crate::tela_html::No::Comentario => None,
        })
        .flat_map(|raiz| std::iter::once(raiz).chain(raiz.descendentes()))
}

fn tem_id_ou_nome(tag: crate::tela_html::Elemento<'_>, valor: &str) -> bool {
    tag.atributo("id").is_some_and(|v| v == valor)
        || tag.atributo("name").is_some_and(|v| v == valor)
}

fn tem_classe(tag: crate::tela_html::Elemento<'_>, classe: &str) -> bool {
    tag.tem_classe(classe)
}

fn algum(
    html: &crate::tela_html::HtmlServido,
    teste: impl Fn(crate::tela_html::Elemento<'_>) -> bool,
) -> bool {
    let dom = crate::tela_html::arvore(html);
    tags(&dom).any(teste)
}

/// Controle de captcha servido (não basta CSS ou função JS de captcha):
/// `txtInfraCaptcha`, ou `g-recaptcha`/`h-captcha` (marcadores do Todos).
pub(crate) fn captcha_exigido(html: &crate::tela_html::HtmlServido) -> bool {
    algum(html, |tag| {
        tem_id_ou_nome(tag, "txtInfraCaptcha")
            || tem_classe(tag, "g-recaptcha")
            || tem_classe(tag, "h-captcha")
    })
}

/// Campo do segundo fator servido (`txtCodigo2FA`, observado pelo Todos).
pub(crate) fn segundo_fator_exigido(html: &crate::tela_html::HtmlServido) -> bool {
    algum(html, |tag| tem_id_ou_nome(tag, "txtCodigo2FA"))
}

/// Texto saneado da mensagem de erro que a tela declara, se houver: a
/// validação da Infra (`txaInfraValidacao`, `UnB` 4.0.9) e, sem ela, os
/// contêineres de recusa.
pub(crate) fn mensagem_de_recusa(html: &crate::tela_html::HtmlServido) -> Option<String> {
    use crate::transporte::texto_de_recusa::texto_visivel;
    if let Some(mensagem) =
        crate::transporte::validacao_da_infra::mensagem_de_validacao_da_infra(html)
    {
        return Some(mensagem);
    }
    let dom = crate::tela_html::arvore(html);
    tags(&dom)
        .filter(|tag| {
            !tem_id_ou_nome(*tag, AVISO_DO_NAVEGADOR)
                && (tem_id_ou_nome(*tag, "divInfraExcecao")
                    || CLASSES_DE_RECUSA.iter().any(|c| tem_classe(*tag, c)))
        })
        .find_map(|tag| crate::transporte::saneamento::sanear(&texto_visivel(tag)))
}

/// URL de `onclick` nas duas formas observadas do menu:
/// `location.href='…'` e `window.location.href='…'`, com `;` opcional.
fn url_do_onclick(onclick: &str) -> Option<&str> {
    let texto = onclick.trim();
    let texto = texto.strip_prefix("window.").unwrap_or(texto);
    let (url, depois) = texto.strip_prefix("location.href='")?.split_once('\'')?;
    (!url.is_empty() && matches!(depois.trim(), "" | ";")).then_some(url)
}

/// URL (relativa, como publicada) do `lnkControleProcessos` do menu.
/// Vários links iguais (barras móvel e padrão) valem como um.
pub(crate) fn link_do_controle(
    html: &crate::tela_html::HtmlServido,
) -> Result<Option<String>, ErroLogin> {
    let dom = crate::tela_html::arvore(html);
    let mut achado: Option<String> = None;
    for h in dom.com_id("lnkControleProcessos") {
        let onclick = h
            .atributo("onclick")
            .map(str::to_owned)
            .ok_or(ErroLogin::LinkDoControleDeProcessosNaoReconhecido)?;
        let url = url_do_onclick(&onclick)
            .ok_or(ErroLogin::LinkDoControleDeProcessosNaoReconhecido)?
            .to_owned();
        if achado.as_ref().is_some_and(|anterior| *anterior != url) {
            return Err(ErroLogin::LinkDoControleDeProcessosNaoReconhecido);
        }
        achado = Some(url);
    }
    Ok(achado)
}

fn atributo_decodificado(
    tag: crate::tela_html::Elemento<'_>,
    nome: &'static str,
) -> Option<String> {
    tag.atributo(nome).map(str::to_owned)
}

/// `onsubmit="return acaoLogin(N);"` com o script servido gravando o
/// argumento em `hdnAcao`: o valor de `hdnAcao` é `N`.
fn hdn_acao_do_onsubmit(
    form: crate::tela_html::Elemento<'_>,
    html: &str,
) -> Result<String, ErroLogin> {
    let onsubmit = form
        .atributo("onsubmit")
        .map(str::to_owned)
        .ok_or(ErroLogin::OnsubmitDoLoginAusente)?;
    let argumento = onsubmit
        .trim()
        .strip_prefix("return acaoLogin(")
        .and_then(|resto| resto.strip_suffix(");"))
        .filter(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
        .ok_or(ErroLogin::OnsubmitDoLoginForaDoFormato)?;
    if !html.contains(HDN_ACAO_PELO_ARGUMENTO) {
        return Err(ErroLogin::ScriptDoLoginSemHdnAcao);
    }
    Ok(argumento.to_owned())
}

/// Texto visível da opção: entidades decodificadas, espaços colapsados.
fn texto_da_opcao(opcao: crate::tela_html::Elemento<'_>) -> String {
    opcao
        .texto()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// `value` da única opção (fora `null`) cujo texto visível ou `value` é
/// igual ao órgão configurado; sem casamento parcial.
fn valor_do_orgao(form: crate::tela_html::Elemento<'_>, orgao: &str) -> Result<String, ErroLogin> {
    let selects: Vec<crate::tela_html::Elemento<'_>> = form
        .elementos("select")
        .filter(|tag| atributo_decodificado(*tag, "name").as_deref() == Some("selOrgao"))
        .collect();
    let select = match selects.as_slice() {
        [] => {
            return Err(ErroLogin::SelecaoDeOrgaoAusenteNaTelaDeLogin);
        }
        [select] => *select,
        _ => {
            return Err(ErroLogin::SelecaoDeOrgaoRepetidaNaTelaDeLogin);
        }
    };
    let mut casados = Vec::new();
    for opcao in select.elementos("option") {
        let texto = texto_da_opcao(opcao);
        // Sem `value`, o navegador envia o texto da opção.
        let valor = atributo_decodificado(opcao, "value").unwrap_or_else(|| texto.clone());
        if valor != "null" && (texto == orgao || valor == orgao) {
            casados.push(valor);
        }
    }
    match casados.as_slice() {
        [] => Err(ErroLogin::OrgaoNaoEncontradoNaTelaDeLogin),
        [valor] => Ok(valor.clone()),
        _ => Err(ErroLogin::OrgaoComMaisDeUmaOpcaoNaTelaDeLogin),
    }
}

/// A `const` do botão servido: `Acessar`, ou `sbmAcessar` com o trecho
/// observado que o desabilita. Outra combinação é recusa.
fn operacao_do_botao(
    form: crate::tela_html::Elemento<'_>,
    html: &str,
) -> Result<&'static OperacaoDeFormulario, ErroLogin> {
    let mut nomes = Vec::new();
    for elemento in ["button", "input"] {
        for tag in form.elementos(elemento) {
            let padrao = if elemento == "button" {
                "submit"
            } else {
                "text"
            };
            let tipo = atributo_decodificado(tag, "type").unwrap_or_else(|| padrao.to_owned());
            if tipo.eq_ignore_ascii_case("submit")
                && let Some(nome) = atributo_decodificado(tag, "name")
                && matches!(nome.as_str(), "Acessar" | "sbmAcessar")
            {
                nomes.push(nome);
            }
        }
    }
    match nomes.as_slice() {
        [nome] if nome == "Acessar" => Ok(&ENTRAR_COM_ACESSAR),
        [nome] if nome == "sbmAcessar" && html.contains(SBM_ACESSAR_DESABILITADO) => {
            Ok(&ENTRAR_COM_SBM_ACESSAR)
        }
        _ => Err(ErroLogin::BotaoDeEnvioDoLoginNaoReconhecido),
    }
}

/// POST do login derivado do `frmLogin` servido, pela `const` do botão
/// observado, com `selOrgao` e `hdnAcao` pelas regras observadas.
pub(crate) fn derivar_login(
    instalacao: &Instalacao,
    tela: &TelaServida<'_>,
    credenciais: &Credenciais,
) -> Result<SubmissaoDerivada, ErroLogin> {
    let dom = crate::tela_html::arvore(tela.html);
    let form = localizar_formulario(&dom, "frmLogin")?;
    let acao = hdn_acao_do_onsubmit(form, tela.html)?;
    let orgao = valor_do_orgao(form, credenciais.orgao())?;
    let operacao = operacao_do_botao(form, tela.html)?;
    let campos = [
        ("txtUsuario", credenciais.usuario()),
        ("pwdSenha", credenciais.senha()),
        ("selOrgao", orgao.as_str()),
        ("hdnAcao", acao.as_str()),
    ];
    derivar_submissao_de_login(operacao, instalacao, tela, &campos).map_err(ErroLogin::Formulario)
}
