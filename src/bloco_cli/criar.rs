//! Prévia Web de criação de bloco de assinatura; deriva o POST sem enviá-lo.
use super::{EnvioDoControle, EnvioHttps, FalhaLeitura, ReferenciasDaSessao};
use crate::Cli;
use crate::acao_do_sei::AcaoDoSei;
use crate::bloco::{self, TipoBloco};
use crate::formulario_web::{
    ConferenciaDaUnidade, EfeitoAvaliado, EnvioDoFormulario, EstadoDoProcesso,
    OperacaoDeFormulario, ResultadoDoEnvio, SubmissaoDerivada, TelaServida, derivar_submissao,
    localizar_formulario,
};
use crate::instalacao::Instalacao;
use crate::pagina_do_controle::codificacao_da_tela;
use crate::tela_html::{HtmlServido, arvore};
use crate::transporte::{self, envio_post::ler_tela_via};

pub(crate) fn link_novo(html: &HtmlServido) -> Result<String, FalhaLeitura> {
    let dom = arvore(html);
    let mut alvo = None;
    // A lista UnB do Todos publica o mesmo btnNovo nas duas barras.
    for botao in dom.com_id("btnNovo") {
        let url = botao
            .atributo("onclick")
            .and_then(crate::sessao_login::tela::url_do_onclick)
            .ok_or(FalhaLeitura::HtmlDeBlocosInvalido)?;
        if alvo.as_deref().is_some_and(|anterior| anterior != url) {
            return Err(FalhaLeitura::HtmlDeBlocosInvalido);
        }
        alvo = Some(url.to_owned());
    }
    alvo.ok_or(FalhaLeitura::HtmlDeBlocosInvalido)
}

pub(crate) fn campo_descricao(html: &HtmlServido) -> Result<&'static str, FalhaLeitura> {
    let dom = arvore(html);
    let form = localizar_formulario(&dom, "frmBlocoCadastro")?;
    let mut campos = form
        .elementos("textarea")
        .filter_map(|no| match no.atributo("name") {
            Some("txtDescricao") if !no.presente("disabled") => Some("txtDescricao"),
            Some("txaDescricao") if !no.presente("disabled") => Some("txaDescricao"),
            _ => None,
        });
    let campo = campos.next().ok_or(FalhaLeitura::HtmlDeBlocosInvalido)?;
    if campos.next().is_some()
        || !form
            .descendentes()
            .any(|no| no.atributo("name") == Some("sbmCadastrarBloco") && !no.presente("disabled"))
    {
        return Err(FalhaLeitura::HtmlDeBlocosInvalido);
    }
    Ok(campo)
}

/// Valor do select simples observado no cadastro; ausência não é valor vazio.
pub(crate) fn grupo_padrao(html: &HtmlServido) -> Result<Option<String>, FalhaLeitura> {
    let dom = arvore(html);
    let form = localizar_formulario(&dom, "frmBlocoCadastro")?;
    let Some(grupo) = form
        .elementos("select")
        .find(|no| no.atributo("name") == Some("selGrupoBloco"))
    else {
        return Ok(None);
    };
    if grupo.presente("disabled") {
        return Ok(None);
    }
    let opcao = grupo
        .elementos("option")
        .filter(|no| no.presente("selected"))
        .last()
        .or_else(|| {
            grupo
                .elementos("option")
                .find(|no| !no.presente("disabled"))
        });
    Ok(opcao.filter(|no| !no.presente("disabled")).map(|no| {
        no.atributo("value")
            .map_or_else(|| no.texto().trim().to_owned(), str::to_owned)
    }))
}

const CRIAR_BLOCO: OperacaoDeFormulario = OperacaoDeFormulario {
    acao: AcaoDoSei::BlocoAssinaturaCadastrar,
    formulario: "frmBlocoCadastro",
    envio: EnvioDoFormulario::BotaoDeEnvio("sbmCadastrarBloco"),
    campos_ocultos_exigidos: &[],
    campos_ocultos_reescritos: &[],
    campos_usuario: &["txtDescricao", "txaDescricao", "selGrupoBloco"],
    resultado: ResultadoDoEnvio::ConferidoPorLeituraPosterior,
    unidade: ConferenciaDaUnidade::NaTelaDeOrigemDoLink,
    efeito: EfeitoAvaliado::AlteraEstadoDoProcesso {
        o_que: EstadoDoProcesso::BlocoCriadoNaUnidade,
    },
};

/// Projeta os campos observados pelo construtor comum, sem tocar na rede.
pub(crate) fn derivar_cadastro(
    instalacao: &Instalacao,
    tela: &TelaServida<'_>,
    unidade: &str,
    descricao: &str,
    grupo: &str,
) -> Result<SubmissaoDerivada, FalhaLeitura> {
    let campo = campo_descricao(tela.html)?;
    Ok(derivar_submissao(
        &CRIAR_BLOCO,
        instalacao,
        tela,
        Some(unidade),
        &[(campo, descricao), ("selGrupoBloco", grupo)],
    )?)
}

fn previa_via<V: EnvioDoControle>(
    sessao: &ReferenciasDaSessao,
    unidade_esperada: Option<&str>,
    descricao: &str,
    envio: &mut V,
) -> Result<serde_json::Value, FalhaLeitura> {
    let controle = ler_tela_via(
        sessao.url_do_controle(),
        sessao.cookie(),
        sessao.origem(),
        |url, cookie| envio.get(url, cookie),
    )?;
    let unidade = crate::controle_reconhecido::unidade_do_controle(&controle.html)?;
    if unidade_esperada
        .is_some_and(|v| v.split_whitespace().collect::<Vec<_>>().join(" ") != unidade)
    {
        return Err(FalhaLeitura::UnidadeInesperada);
    }
    let url_lista = bloco::link_do_menu(
        &controle.instalacao,
        &controle.url,
        &controle.html,
        TipoBloco::Assinatura,
    )?;
    let cookie = sessao.cookie().valor()?;
    let lista = transporte::decodificar_resposta(envio.get(&url_lista, cookie)?)?;
    let url = bloco::resolver_link(
        &controle.instalacao,
        &url_lista,
        &link_novo(&lista)?,
        AcaoDoSei::BlocoAssinaturaCadastrar,
        None,
    )?;
    let resposta = envio.get(&url, cookie)?;
    let codificacao =
        codificacao_da_tela(&resposta.mime).ok_or(FalhaLeitura::HtmlDeBlocosInvalido)?;
    let cadastro = transporte::decodificar_resposta(resposta)?;
    let grupo = grupo_padrao(&cadastro)?.ok_or(FalhaLeitura::HtmlDeBlocosInvalido)?;
    let tela = TelaServida {
        url: &url,
        html: &cadastro,
        codificacao,
    };
    // A prévia também confronta a action, o charset e os controles servidos.
    // A submissão derivada é descartada; nenhum POST nesta fatia.
    drop(derivar_cadastro(
        &controle.instalacao,
        &tela,
        &unidade,
        descricao,
        &grupo,
    )?);
    Ok(serde_json::json!({
        "unidade": unidade,
        "tipo_bloco": TipoBloco::Assinatura,
        "descricao": descricao,
        "grupo_padrao": grupo,
        "confirmado": false,
        "alterado": false
    }))
}

pub(super) fn executar(cli: &Cli, descricao: &str) -> anyhow::Result<()> {
    anyhow::ensure!(!descricao.trim().is_empty(), "descrição do bloco vazia");
    let arquivos = crate::configuracao::arquivos_de_acesso()?;
    let sessao =
        ReferenciasDaSessao::ler(&arquivos.url_controle, &arquivos.origem, &arquivos.cookie)?;
    let previa = previa_via(&sessao, cli.unidade.as_deref(), descricao, &mut EnvioHttps)?;
    if cli.json {
        println!("{}", serde_json::to_string_pretty(&previa)?);
    } else {
        println!("IES: prévia de novo bloco de assinatura: {descricao}. Nada gravado.");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn previa_le_grupo_servido_sem_confundir_vazio_e_ausencia() {
        let casos = [
            (
                "<select name='selGrupoBloco'><option value='null'>Nenhum</option></select>",
                Some("null"),
            ),
            (
                "<select name='selGrupoBloco'><option value='null'>Nenhum</option><option selected value=''>Todos</option></select>",
                Some(""),
            ),
            (
                "<select name='selGrupoBloco'><option value='null'>Nenhum</option><option selected value='4'>Grupo</option></select>",
                Some("4"),
            ),
            ("<textarea name='txtDescricao'></textarea>", None),
        ];
        for (controles, esperado) in casos {
            let html =
                HtmlServido::de_teste(&format!("<form id='frmBlocoCadastro'>{controles}</form>"));
            assert_eq!(grupo_padrao(&html).unwrap().as_deref(), esperado);
        }
    }

    #[test]
    fn derivacao_preserva_campo_da_versao_grupo_oculto_e_charset() {
        use crate::formulario_web::CodificacaoDaTela;

        let origem = "https://sei.exemplo.gov.br/sei";
        let instalacao = Instalacao::interpretar(origem).unwrap();
        let hash = "a".repeat(64);
        let action = format!("controlador.php?acao=bloco_assinatura_cadastrar&infra_hash={hash}");
        let url = format!("{origem}/{action}");
        let casos = [
            (
                "txtDescricao",
                CodificacaoDaTela::Iso88591,
                "A%E7%E3o+%26+B",
            ),
            (
                "txaDescricao",
                CodificacaoDaTela::Utf8,
                "A%C3%A7%C3%A3o+%26+B",
            ),
        ];
        for (campo, codificacao, codificado) in casos {
            let html = HtmlServido::de_teste(&format!(
                "<form id='frmBlocoCadastro' method='post' action='{action}'>\
                <input type='hidden' name='hdnExemplo' value='original'>\
                <textarea name='{campo}'></textarea>\
                <select name='selGrupoBloco'><option value='null'>Nenhum</option></select>\
                <button type='submit' name='sbmCadastrarBloco' value='Salvar'>Salvar</button></form>"
            ));
            let tela = TelaServida {
                url: &url,
                html: &html,
                codificacao,
            };
            let submissao =
                derivar_cadastro(&instalacao, &tela, "TESTE", "Ação & B", "null").unwrap();
            assert_eq!(
                std::str::from_utf8(submissao.corpo()).unwrap(),
                format!(
                    "hdnExemplo=original&{campo}={codificado}&selGrupoBloco=null&sbmCadastrarBloco=Salvar"
                )
            );
            assert_eq!(submissao.url(), url);
        }
    }

    #[test]
    fn novo_duplicado_so_quando_mesmo_destino() {
        let botao = "<button id='btnNovo' onclick=\"location.href='controlador.php?acao=bloco_assinatura_cadastrar'\">Novo</button>";
        let html = HtmlServido::de_teste(&format!("{botao}{botao}"));
        assert_eq!(
            link_novo(&html).unwrap(),
            "controlador.php?acao=bloco_assinatura_cadastrar"
        );
        let outro = botao.replace("cadastrar", "listar");
        let html = HtmlServido::de_teste(&format!("{botao}{outro}"));
        assert!(link_novo(&html).is_err());
        assert!(link_novo(&HtmlServido::de_teste("<html></html>")).is_err());
    }
}
