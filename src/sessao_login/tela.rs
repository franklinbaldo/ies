//! Telas do login, lidas com `tl` e `html_atributos` (sem regex): o
//! formulário `frmLogin` servido, captcha e segundo fator exigidos, a
//! mensagem de recusa e o link do Controle no Painel. As `const` da
//! operação ficam aqui; o primitivo de formulário deriva o POST.
use super::{Credenciais, ErroLogin};
use crate::formulario_web::{
    ConferenciaDaUnidade, EfeitoAvaliado, EnvioDoFormulario, EstadoDeSessao, OperacaoDeFormulario,
    ResultadoDoEnvio, SubmissaoDerivada, TelaServida, decodificar_atributo,
    derivar_submissao_de_login, localizar_formulario,
};
use crate::html_atributos::{atributos, nos_com_id, valor_do_atributo};
use crate::instalacao::Instalacao;

const CAMPOS_DO_LOGIN: &[&str] = &["txtUsuario", "pwdSenha", "selOrgao"];

/// `hdnAcao` é gravado pelo `acaoLogin(N)` do `onsubmit` servido.
const OCULTOS_REESCRITOS: &[&str] = &["hdnAcao"];

/// Login com o botão `Acessar` (`UnB` 4.0.9): o `acaoLogin(2)` servido não o
/// desabilita, e ele participa do corpo com o valor servido.
pub(crate) const ENTRAR_COM_ACESSAR: OperacaoDeFormulario = OperacaoDeFormulario {
    acao: "login",
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

fn handle(indice: usize) -> tl::NodeHandle {
    tl::NodeHandle::new(u32::try_from(indice).unwrap_or(u32::MAX))
}

/// Tags legíveis do documento, em ordem; tag ilegível é ignorada aqui
/// (só serve para afirmar presença, nunca ausência de recusa).
fn tags<'d>(dom: &'d tl::VDom<'d>) -> impl Iterator<Item = (tl::NodeHandle, &'d tl::HTMLTag<'d>)> {
    let parser = dom.parser();
    (0..dom.nodes().len()).filter_map(move |i| {
        let h = handle(i);
        h.get(parser).and_then(tl::Node::as_tag).map(|tag| (h, tag))
    })
}

fn tem_id_ou_nome(tag: &tl::HTMLTag<'_>, valor: &str) -> bool {
    atributos(tag)
        .is_ok_and(|a| a.valores("id").any(|v| v == valor) || a.valores("name").any(|v| v == valor))
}

fn tem_classe(tag: &tl::HTMLTag<'_>, classe: &str) -> bool {
    atributos(tag).is_ok_and(|a| a.tem_classe(classe).unwrap_or(false))
}

fn algum(html: &str, teste: impl Fn(&tl::HTMLTag<'_>) -> bool) -> bool {
    tl::parse(html, tl::ParserOptions::default())
        .is_ok_and(|dom| tags(&dom).any(|(_, tag)| teste(tag)))
}

/// Controle de captcha servido (não basta CSS ou função JS de captcha):
/// `txtInfraCaptcha`, ou `g-recaptcha`/`h-captcha` (marcadores do Todos).
pub(crate) fn captcha_exigido(html: &str) -> bool {
    algum(html, |tag| {
        tem_id_ou_nome(tag, "txtInfraCaptcha")
            || tem_classe(tag, "g-recaptcha")
            || tem_classe(tag, "h-captcha")
    })
}

/// Campo do segundo fator servido (`txtCodigo2FA`, observado pelo Todos).
pub(crate) fn segundo_fator_exigido(html: &str) -> bool {
    algum(html, |tag| tem_id_ou_nome(tag, "txtCodigo2FA"))
}

/// Texto saneado da mensagem de erro que a tela declara, se houver: a
/// validação da Infra (`txaInfraValidacao`, `UnB` 4.0.9) e, sem ela, os
/// contêineres de recusa.
pub(crate) fn mensagem_de_recusa(html: &str) -> Option<String> {
    use crate::transporte::texto_de_recusa::texto_visivel;
    if let Some(mensagem) =
        crate::transporte::validacao_da_infra::mensagem_de_validacao_da_infra(html)
    {
        return Some(mensagem);
    }
    let dom = tl::parse(html, tl::ParserOptions::default()).ok()?;
    tags(&dom)
        .filter(|(_, tag)| {
            !tem_id_ou_nome(tag, AVISO_DO_NAVEGADOR)
                && (tem_id_ou_nome(tag, "divInfraExcecao")
                    || CLASSES_DE_RECUSA.iter().any(|c| tem_classe(tag, c)))
        })
        .find_map(|(h, _)| {
            let texto =
                html_escape::decode_html_entities(&texto_visivel(h, dom.parser())).into_owned();
            crate::transporte::saneamento::sanear(&texto)
        })
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
pub(crate) fn link_do_controle(html: &str) -> Result<Option<String>, ErroLogin> {
    let dom = tl::parse(html, tl::ParserOptions::default())
        .map_err(|_| ErroLogin::LinkDoControleDeProcessosNaoReconhecido)?;
    let parser = dom.parser();
    let mut achado: Option<String> = None;
    for h in nos_com_id(&dom, "lnkControleProcessos")
        .map_err(|_| ErroLogin::LinkDoControleDeProcessosNaoReconhecido)?
    {
        let tag = h
            .get(parser)
            .and_then(tl::Node::as_tag)
            .ok_or(ErroLogin::LinkDoControleDeProcessosNaoReconhecido)?;
        let bruto = valor_do_atributo(tag, "onclick")
            .map_err(|_| ErroLogin::LinkDoControleDeProcessosNaoReconhecido)?
            .ok_or(ErroLogin::LinkDoControleDeProcessosNaoReconhecido)?;
        let onclick = decodificar_atributo(&bruto)
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

fn atributo_decodificado(tag: &tl::HTMLTag<'_>, nome: &'static str) -> Option<String> {
    valor_do_atributo(tag, nome)
        .ok()
        .flatten()
        .and_then(|bruto| decodificar_atributo(&bruto))
}

/// `onsubmit="return acaoLogin(N);"` com o script servido gravando o
/// argumento em `hdnAcao`: o valor de `hdnAcao` é `N`.
fn hdn_acao_do_onsubmit(form: &tl::HTMLTag<'_>, html: &str) -> Result<String, ErroLogin> {
    let onsubmit = atributo_decodificado(form, "onsubmit").unwrap_or_default();
    onsubmit
        .trim()
        .strip_prefix("return acaoLogin(")
        .and_then(|resto| resto.strip_suffix(");"))
        .filter(|n| (1..=3).contains(&n.len()) && n.bytes().all(|b| b.is_ascii_digit()))
        .filter(|_| html.contains(HDN_ACAO_PELO_ARGUMENTO))
        .map(str::to_owned)
        .ok_or(ErroLogin::RegraDoOnsubmitDoLoginNaoObservada)
}

/// Texto visível da opção: entidades decodificadas, espaços colapsados.
fn texto_da_opcao(opcao: &tl::HTMLTag<'_>, parser: &tl::Parser<'_>) -> String {
    html_escape::decode_html_entities(&opcao.inner_text(parser))
        .into_owned()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// `value` da única opção (fora `null`) cujo texto visível ou `value` é
/// igual ao órgão configurado; sem casamento parcial.
fn valor_do_orgao(
    form: &tl::HTMLTag<'_>,
    parser: &tl::Parser<'_>,
    orgao: &str,
) -> Result<String, ErroLogin> {
    let selects: Vec<&tl::HTMLTag<'_>> = form
        .query_selector(parser, "select")
        .into_iter()
        .flatten()
        .filter_map(|h| h.get(parser).and_then(tl::Node::as_tag))
        .filter(|tag| atributo_decodificado(tag, "name").as_deref() == Some("selOrgao"))
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
    for opcao in select
        .query_selector(parser, "option")
        .into_iter()
        .flatten()
        .filter_map(|h| h.get(parser).and_then(tl::Node::as_tag))
    {
        let texto = texto_da_opcao(opcao, parser);
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
    form: &tl::HTMLTag<'_>,
    parser: &tl::Parser<'_>,
    html: &str,
) -> Result<&'static OperacaoDeFormulario, ErroLogin> {
    let mut nomes = Vec::new();
    for elemento in ["button", "input"] {
        for tag in form
            .query_selector(parser, elemento)
            .into_iter()
            .flatten()
            .filter_map(|h| h.get(parser).and_then(tl::Node::as_tag))
        {
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
    let dom = tl::parse(tela.html, tl::ParserOptions::default())
        .map_err(|_| ErroLogin::TelaDeLoginNaoReconhecida)?;
    let parser = dom.parser();
    let form = localizar_formulario(&dom, "frmLogin")?;
    let acao = hdn_acao_do_onsubmit(form, tela.html)?;
    let orgao = valor_do_orgao(form, parser, credenciais.orgao())?;
    let operacao = operacao_do_botao(form, parser, tela.html)?;
    let campos = [
        ("txtUsuario", credenciais.usuario()),
        ("pwdSenha", credenciais.senha()),
        ("selOrgao", orgao.as_str()),
        ("hdnAcao", acao.as_str()),
    ];
    derivar_submissao_de_login(operacao, instalacao, tela, &campos).map_err(ErroLogin::Formulario)
}
