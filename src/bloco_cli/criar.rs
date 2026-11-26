//! Criação Web de bloco de assinatura: prévia, POST único e releitura.
use super::{
    EnvioDoControle, EnvioHttps, FalhaLeitura, Leitura, ReferenciasDaSessao, ler_tipado_via,
};
use crate::Cli;
use crate::acao_do_sei::AcaoDoSei;
use crate::bloco::{self, TipoBloco};
use crate::bloco_estado::EstadoDoBloco;
use crate::consulta_do_controlador::ConsultaDoControlador;
use crate::formulario_web::{
    ConferenciaDaUnidade, EfeitoAvaliado, EnvioDoFormulario, EstadoDoProcesso,
    OperacaoDeFormulario, RespostaDoEnvio, ResultadoDoEnvio, SubmissaoDerivada, TelaServida,
    consulta_do_controlador, derivar_submissao, enviar_submissao_via, localizar_formulario,
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

pub(crate) fn id_do_aceite<'a>(local: &'a str, url: &str) -> Option<&'a str> {
    let consulta = ConsultaDoControlador::ler(consulta_do_controlador(local, url)?).ok()?;
    if consulta.acao().ok()? != AcaoDoSei::BlocoAssinaturaListar.como_na_url() {
        return None;
    }
    let id = consulta.unico("id_bloco").ok()?;
    (!id.is_empty() && id.bytes().all(|b| b.is_ascii_digit())).then_some(id)
}

fn conferir_criacao<'a>(
    antes: &bloco::ListaDeBlocos,
    depois: &'a bloco::ListaDeBlocos,
    id: &str,
    descricao: &str,
) -> Option<&'a bloco::BlocoLido> {
    if antes.itens.iter().any(|b| b.numero == id) {
        return None;
    }
    let descricao_visivel = descricao
        .replace('\u{a0}', " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    depois.itens.iter().find(|b| {
        b.numero == id && b.estado == EstadoDoBloco::Gerado && b.descricao == descricao_visivel
    })
}

pub(super) fn criar_via<V: EnvioDoControle>(
    sessao: &ReferenciasDaSessao,
    unidade_esperada: Option<&str>,
    descricao: &str,
    confirmar: bool,
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
    let submissao = derivar_cadastro(&controle.instalacao, &tela, &unidade, descricao, &grupo)?;
    if confirmar {
        let antes = bloco::ler_lista(&lista, TipoBloco::Assinatura)?.lista;
        let resposta = enviar_submissao_via(submissao, sessao.cookie(), |sub, cookie| {
            envio.post(sub, cookie)
        });
        let Ok(RespostaDoEnvio::Redirecionamento(r)) = resposta else {
            return Err(FalhaLeitura::NaoConfirmado);
        };
        let id = r
            .localizacao()
            .and_then(|l| id_do_aceite(l, r.url_do_envio()));
        let relido = ler_tipado_via(
            sessao,
            Some(&unidade),
            TipoBloco::Assinatura,
            None,
            None,
            envio,
        )
        .map_err(|_| FalhaLeitura::NaoConfirmado)?;
        let Leitura::Lista(depois) = relido.leitura else {
            return Err(FalhaLeitura::NaoConfirmado);
        };
        let id = id.ok_or(FalhaLeitura::NaoConfirmado)?;
        let bloco =
            conferir_criacao(&antes, &depois, id, descricao).ok_or(FalhaLeitura::NaoConfirmado)?;
        return Ok(serde_json::json!({
            "unidade": unidade,
            "tipo_bloco": TipoBloco::Assinatura,
            "descricao": descricao,
            "grupo_padrao": grupo,
            "bloco": bloco,
            "confirmado": true,
            "alterado": true
        }));
    }
    Ok(serde_json::json!({
        "unidade": unidade,
        "tipo_bloco": TipoBloco::Assinatura,
        "descricao": descricao,
        "grupo_padrao": grupo,
        "confirmado": false,
        "alterado": false
    }))
}

pub(super) fn executar(cli: &Cli, descricao: &str, confirmar: bool) -> anyhow::Result<()> {
    anyhow::ensure!(!descricao.trim().is_empty(), "descrição do bloco vazia");
    let arquivos = crate::configuracao::arquivos_de_acesso()?;
    let sessao =
        ReferenciasDaSessao::ler(&arquivos.url_controle, &arquivos.origem, &arquivos.cookie)?;
    if confirmar {
        crate::origem_de_treinamento::exigir(
            crate::origem_de_treinamento::Escrita::BlocoCriar,
            sessao.origem(),
        )?;
    }
    let previa = criar_via(
        &sessao,
        cli.unidade.as_deref(),
        descricao,
        confirmar,
        &mut EnvioHttps,
    )?;
    if cli.json {
        println!("{}", serde_json::to_string_pretty(&previa)?);
    } else if confirmar {
        println!("IES: bloco de assinatura criado e conferido: {descricao}.");
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
    fn aceite_publica_identidade_unica_na_lista_da_mesma_origem() {
        let url = "https://sei.exemplo.gov.br/sei/controlador.php?acao=bloco_assinatura_cadastrar";
        assert_eq!(
            id_do_aceite(
                "controlador.php?acao=bloco_assinatura_listar&id_bloco=42",
                url
            ),
            Some("42")
        );
        for local in [
            "controlador.php?acao=bloco_assinatura_listar",
            "controlador.php?acao=bloco_assinatura_listar&id_bloco=42&id_bloco=43",
            "controlador.php?acao=bloco_assinatura_listar&id_bloco=x",
            "controlador.php?acao=bloco_interno_listar&id_bloco=42",
            "https://outro.example/sei/controlador.php?acao=bloco_assinatura_listar&id_bloco=42",
        ] {
            assert!(id_do_aceite(local, url).is_none());
        }
    }

    #[test]
    fn releitura_exige_id_novo_estado_e_descricao() {
        let linha = |numero: &str, estado, descricao: &str| bloco::BlocoLido {
            numero: numero.to_owned(),
            tipo: TipoBloco::Assinatura,
            estado,
            descricao: descricao.to_owned(),
        };
        let lista = |itens: Vec<bloco::BlocoLido>| bloco::ListaDeBlocos {
            tipo: TipoBloco::Assinatura,
            total_declarado: itens.len(),
            faixa_observada: None,
            itens,
        };
        let antiga = linha("41", EstadoDoBloco::Gerado, "Mesmo texto");
        let antes = lista(vec![antiga.clone()]);
        let nova = linha("42", EstadoDoBloco::Gerado, "Mesmo texto");
        let depois = lista(vec![antiga.clone(), nova.clone()]);
        assert_eq!(
            conferir_criacao(&antes, &depois, "42", "Mesmo  texto"),
            Some(&nova)
        );
        // Cadastro vazio observado também é completo (0/0).
        let vazio = lista(Vec::new());
        assert_eq!(
            conferir_criacao(&vazio, &depois, "42", "Mesmo texto"),
            Some(&nova)
        );

        // A releitura só precisa mostrar a identidade procurada; não
        // requer que todos os demais blocos estejam na mesma página.
        let mut posterior_parcial = lista(vec![nova.clone()]);
        posterior_parcial.total_declarado = 2;
        posterior_parcial.faixa_observada = Some(crate::legenda::Faixa { inicio: 2, fim: 2 });
        assert_eq!(
            conferir_criacao(&antes, &posterior_parcial, "42", "Mesmo texto"),
            Some(&nova)
        );

        assert!(conferir_criacao(&antes, &depois, "41", "Mesmo texto").is_none());
        assert!(conferir_criacao(&antes, &depois, "43", "Mesmo texto").is_none());
        assert!(conferir_criacao(&antes, &depois, "42", "Outro texto").is_none());
        let errado = lista(vec![linha("42", EstadoDoBloco::Recebido, "Mesmo texto")]);
        assert!(conferir_criacao(&antes, &errado, "42", "Mesmo texto").is_none());
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
