//! Leitura GET do catálogo de marcadores publicado pelo Controle (#880).
use crate::acao_do_sei::AcaoDoSei;
use crate::colheita_do_controle::Colheita;
use crate::consulta_do_controlador::{ConsultaDoControlador, menciona_acao};
use crate::indice_local::{self, IndiceLocal};
use crate::instalacao::Instalacao;
use crate::marcadores_unidade::{self, ErroCatalogo, Estado};
use crate::tela_html::{HtmlServido, arvore};
use crate::transporte::{self, FalhaTransporte, sessao::ReferenciasDaSessao};
use crate::url_observada::{UrlHttps, pasta};
use crate::{configuracao, controle_reconhecido, relogin_cli};
use serde::Serialize;

#[derive(Debug, thiserror::Error)]
#[error("{self:?}")]
enum FalhaLeitura {
    Transporte(#[from] FalhaTransporte),
    Controle(#[from] crate::erro_web::ErroCaixa),
    Catalogo(#[from] ErroCatalogo),
    UnidadeInesperada,
    LinkAusente,
    LinkAmbiguo,
    LinkInvalido,
    Consulta(#[from] crate::consulta_do_controlador::ErroConsultaDoControlador),
}

impl FalhaLeitura {
    fn sessao_encerrada(&self) -> bool {
        matches!(self, Self::Transporte(erro) if erro.sessao_encerrada())
    }
}

/// A janela derivada pode omitir a barra; se a publicar, deve confirmar a unidade.
fn conferir_unidade_da_lista(html: &HtmlServido, esperada: &str) -> Result<(), FalhaLeitura> {
    let dom = arvore(html);
    if dom.com_id("lnkInfraUnidade").next().is_some()
        && crate::controle_contexto::unidade_html(&dom)?.as_deref() != Some(esperada)
    {
        return Err(FalhaLeitura::UnidadeInesperada);
    }
    Ok(())
}

/// Só o link da ação publicado pelo Controle; nunca inventa hash ou rota.
fn link_do_menu(
    html: &HtmlServido,
    servida: &str,
    instalacao: &Instalacao,
) -> Result<String, FalhaLeitura> {
    let dom = arvore(html);
    let mut links = dom
        .elementos("a")
        .filter_map(|no| no.atributo("href"))
        .filter(|href| menciona_acao(href, AcaoDoSei::MarcadorListar));
    let href = links.next().ok_or(FalhaLeitura::LinkAusente)?;
    if links.next().is_some() {
        return Err(FalhaLeitura::LinkAmbiguo);
    }
    url_publicada(href, servida, instalacao, AcaoDoSei::MarcadorListar)
}

fn url_publicada(
    href: &str,
    servida: &str,
    instalacao: &Instalacao,
    acao: AcaoDoSei,
) -> Result<String, FalhaLeitura> {
    let consulta = href
        .strip_prefix("controlador.php?")
        .ok_or(FalhaLeitura::LinkInvalido)?;
    let consulta = ConsultaDoControlador::ler(consulta)?;
    if consulta.acao() != Ok(acao.como_na_url())
        || consulta.valores("acao_ajax").next().is_some()
        || !consulta.sem_chaves_repetidas()
        || !consulta
            .unico("infra_hash")
            .is_ok_and(|hash| hash.len() == 64 && hash.bytes().all(|b| b.is_ascii_hexdigit()))
    {
        return Err(FalhaLeitura::LinkInvalido);
    }
    let base = UrlHttps::interpretar(servida).ok_or(FalhaLeitura::LinkInvalido)?;
    let dir = pasta(base.caminho).ok_or(FalhaLeitura::LinkInvalido)?;
    let url = format!("https://{}/{dir}/{href}", base.host);
    let alvo = UrlHttps::interpretar(&url).ok_or(FalhaLeitura::LinkInvalido)?;
    if !instalacao.autoriza_controlador(alvo.host, alvo.caminho) {
        return Err(FalhaLeitura::LinkInvalido);
    }
    Ok(url)
}

#[derive(Serialize)]
struct Saida {
    unidade: String,
    origem: &'static str,
    requisicoes: u8,
    #[serde(flatten)]
    selecao: marcadores_unidade::Selecao,
}

pub(super) fn listar(unidade: Option<&str>, json_mode: bool, inativos: bool) -> anyhow::Result<()> {
    let estado = if inativos {
        Estado::Inativo
    } else {
        Estado::Ativo
    };
    let arquivos = configuracao::arquivos_de_acesso()?;
    let indice = IndiceLocal::da_configuracao(&arquivos);
    let ((atual, selecao), nota) = relogin_cli::ler(
        &arquivos,
        &indice,
        unidade,
        FalhaLeitura::sessao_encerrada,
        || {
            let sessao = ReferenciasDaSessao::ler(
                &arquivos.url_controle,
                &arquivos.origem,
                &arquivos.cookie,
            )?;
            let controle = transporte::envio_post::ler_tela_do_controle(&sessao)?;
            let atual = controle_reconhecido::unidade_do_controle(&controle.html)?;
            if unidade.is_some_and(|esperada| esperada != atual) {
                return Err(FalhaLeitura::UnidadeInesperada);
            }
            Colheita::das_referencias(&indice, &sessao)
                .colher(&controle.html, Some(&controle.mime));
            let url = link_do_menu(&controle.html, &controle.url, &controle.instalacao)?;
            let resposta = transporte::get_https(&url, sessao.cookie().valor()?)?;
            let html = transporte::decodificar_resposta(resposta)?;
            conferir_unidade_da_lista(&html, &atual)?;
            let selecao = marcadores_unidade::ler(&html)?.selecionar(estado);
            Ok((atual, selecao))
        },
    )
    .map_err(relogin_cli::subir)?;
    let requisicoes = relogin_cli::somar(2, nota.as_ref());
    if json_mode {
        let mut valor = serde_json::to_value(Saida {
            unidade: atual,
            origem: "web",
            requisicoes,
            selecao,
        })?;
        relogin_cli::anotar(&mut valor, nota.as_ref());
        indice_local::anexar_avisos(&mut valor, &indice);
        println!("{valor}");
    } else {
        relogin_cli::avisar(false, nota.as_ref());
        indice_local::avisar_no_texto(&indice);
        println!(
            "Marcadores da unidade {atual}: {} selecionado(s) de {} observado(s); {} declarados no catálogo; {requisicoes} requisição(ões).",
            selecao.itens.len(),
            selecao.total_observado,
            selecao.total_declarado
        );
        let estado = match selecao.estado {
            Estado::Ativo => "ativo",
            Estado::Inativo => "inativo",
        };
        println!("Estado solicitado: {estado}");
        if let Some(faixa) = &selecao.faixa_observada {
            println!("Faixa observada: {} a {}", faixa.inicio, faixa.fim);
        }
        for item in selecao.itens {
            println!("{}#{} | {}", item.id, item.nome, item.cor);
        }
    }
    Ok(())
}


#[derive(Debug, thiserror::Error)]
#[error("{self:?}")]
enum FalhaPrevia {
    ConfirmacaoIndisponivel,
    NomeInvalido,
    DescricaoInvalida,
    NomeJaObservado,
    FormularioNaoReconhecido,
    CorNaoOferecidaOuAmbigua,
}

fn link_novo(
    html: &HtmlServido,
    servida: &str,
    instalacao: &Instalacao,
) -> Result<String, FalhaLeitura> {
    let dom = arvore(html);
    let mut links = dom.com_id("btnNovo").map(|botao| {
        if botao.nome() != "button" || botao.atributo("type") != Some("button") {
            return Err(FalhaLeitura::LinkInvalido);
        }
        botao
            .atributo("onclick")
            .and_then(crate::sessao_login::tela::url_do_onclick)
            .ok_or(FalhaLeitura::LinkInvalido)
    });
    let href = links.next().ok_or(FalhaLeitura::LinkAusente)??;
    // A lista real repete o mesmo botão no topo e no rodapé.
    for link in links {
        if link? != href {
            return Err(FalhaLeitura::LinkAmbiguo);
        }
    }
    url_publicada(href, servida, instalacao, AcaoDoSei::MarcadorCadastrar)
}

fn cor_do_formulario(
    html: &HtmlServido,
    servida: &str,
    instalacao: &Instalacao,
    pedida: &str,
) -> anyhow::Result<String> {
    let dom = arvore(html);
    let mut formularios = dom.com_id("frmMarcadorCadastro");
    let form = formularios
        .next()
        .ok_or(FalhaPrevia::FormularioNaoReconhecido)?;
    if formularios.next().is_some()
        || form.nome() != "form"
        || form.atributo("method") != Some("post")
    {
        return Err(FalhaPrevia::FormularioNaoReconhecido.into());
    }
    url_publicada(
        form.atributo("action")
            .ok_or(FalhaPrevia::FormularioNaoReconhecido)?,
        servida,
        instalacao,
        AcaoDoSei::MarcadorCadastrar,
    )?;
    for (tag, campo) in [
        ("input", "txtNome"),
        ("textarea", "txaDescricao"),
        ("input", "hdnStaIcone"),
        ("button", "sbmCadastrarMarcador"),
    ] {
        if form
            .elementos(tag)
            .filter(|e| e.atributo("name") == Some(campo))
            .count()
            != 1
        {
            return Err(FalhaPrevia::FormularioNaoReconhecido.into());
        }
    }
    let mut seletores = form
        .elementos("select")
        .filter(|e| e.atributo("name") == Some("selStaIcone"));
    let seletor = seletores
        .next()
        .ok_or(FalhaPrevia::FormularioNaoReconhecido)?;
    if seletores.next().is_some() {
        return Err(FalhaPrevia::FormularioNaoReconhecido.into());
    }
    let mut cores = seletor
        .elementos("option")
        .filter(|e| e.texto().trim() == pedida)
        .filter_map(|e| e.atributo("value"))
        .filter(|v| !v.is_empty() && v.bytes().all(|b| b.is_ascii_digit()));
    let codigo = cores.next().ok_or(FalhaPrevia::CorNaoOferecidaOuAmbigua)?;
    if cores.next().is_some() {
        return Err(FalhaPrevia::CorNaoOferecidaOuAmbigua.into());
    }
    Ok(codigo.to_owned())
}

#[derive(Serialize)]
struct Previa<'a> {
    unidade: &'a str,
    nome: &'a str,
    cor: &'a str,
    codigo_cor: String,
    descricao: &'a str,
    previa: bool,
    criado: bool,
    requisicoes: u8,
}

/// Prévia não abre sessão nova, não alterna filtro e não submete formulário.
pub(super) fn criar(
    unidade: Option<&str>,
    json_mode: bool,
    nome: &str,
    cor: &str,
    descricao: &str,
    confirmar: bool,
) -> anyhow::Result<()> {
    if confirmar {
        return Err(FalhaPrevia::ConfirmacaoIndisponivel.into());
    }
    let nome = nome.trim();
    if nome.is_empty() || nome.encode_utf16().count() > 50 {
        return Err(FalhaPrevia::NomeInvalido.into());
    }
    if descricao.encode_utf16().count() > 250 {
        return Err(FalhaPrevia::DescricaoInvalida.into());
    }
    let arquivos = configuracao::arquivos_de_acesso()?;
    let sessao = ReferenciasDaSessao::ler(
        &arquivos.url_controle,
        &arquivos.origem,
        &arquivos.cookie,
    )?;
    let controle = transporte::envio_post::ler_tela_do_controle(&sessao)?;
    let atual = controle_reconhecido::unidade_do_controle(&controle.html)?;
    if unidade.is_some_and(|esperada| esperada != atual) {
        return Err(FalhaLeitura::UnidadeInesperada.into());
    }
    let url = link_do_menu(&controle.html, &controle.url, &controle.instalacao)?;
    let resposta = transporte::get_https(&url, sessao.cookie().valor()?)?;
    let lista = transporte::decodificar_resposta(resposta)?;
    conferir_unidade_da_lista(&lista, &atual)?;
    let observada = marcadores_unidade::ler(&lista)?;
    if observada
        .itens
        .iter()
        .any(|m| m.estado == Estado::Ativo && m.nome.to_lowercase() == nome.to_lowercase())
    {
        return Err(FalhaPrevia::NomeJaObservado.into());
    }
    let novo = link_novo(&lista, &url, &controle.instalacao)?;
    let resposta = transporte::get_https(&novo, sessao.cookie().valor()?)?;
    let formulario = transporte::decodificar_resposta(resposta)?;
    if crate::controle_contexto::unidade_html(&arvore(&formulario))?.as_deref()
        != Some(atual.as_str())
    {
        return Err(FalhaLeitura::UnidadeInesperada.into());
    }
    let codigo = cor_do_formulario(&formulario, &novo, &controle.instalacao, cor)?;
    if json_mode {
        println!(
            "{}",
            serde_json::to_string(&Previa {
                unidade: &atual,
                nome,
                cor,
                codigo_cor: codigo,
                descricao,
                previa: true,
                criado: false,
                requisicoes: 3,
            })?
        );
    } else {
        println!("Prévia: criar marcador {nome} na unidade {atual}, cor {cor} (código {codigo}).");
        println!("Descrição: {descricao}");
        println!("Nenhum marcador criado; confirmação indisponível.");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::*;

    #[test]
    fn menu_ausente_ambiguo_e_origem_alheia() {
        let origem = "https://sei.example/sei";
        let instalacao = Instalacao::interpretar(origem).unwrap();
        let servida = format!("{origem}/controlador.php?acao=procedimento_controlar");
        let href = format!(
            "controlador.php?acao=marcador_listar&infra_hash={}",
            "a".repeat(64)
        );
        let link = format!("<a href='{href}'>Marcadores</a>");
        let ler = |html: &str| link_do_menu(&HtmlServido::de_teste(html), &servida, &instalacao);
        assert_eq!(ler(&link).unwrap(), format!("{origem}/{href}"));
        assert!(matches!(
            ler("<p>Controle</p>"),
            Err(FalhaLeitura::LinkAusente)
        ));
        assert!(matches!(
            ler(&format!("{link}{link}")),
            Err(FalhaLeitura::LinkAmbiguo)
        ));
        assert!(matches!(
            ler(&format!(
                "<a href='https://outra.example/sei/{href}'>Marcadores</a>"
            )),
            Err(FalhaLeitura::LinkInvalido)
        ));
    }

    #[test]
    fn contexto_publicado_da_lista_confirma_unidade_do_controle() {
        let conferir = |html: &str| conferir_unidade_da_lista(&HtmlServido::de_teste(html), "A");
        for html in [
            "<p>Janela sem barra</p>",
            "<a id='lnkInfraUnidade'>A</a>",
            "<div id='divInfraBarraSistemaMovel'><a id='lnkInfraUnidade' href='#'>A</a></div>\
             <div id='divInfraBarraSistemaPadrao'><a id='lnkInfraUnidade' href='#'>A</a></div>",
        ] {
            assert!(conferir(html).is_ok(), "{html}");
        }
        for html in [
            "<a id='lnkInfraUnidade'>B</a>",
            "<a id='lnkInfraUnidade'> </a>",
            "<span id='lnkInfraUnidade'>A</span>",
        ] {
            assert!(matches!(
                conferir(html),
                Err(FalhaLeitura::UnidadeInesperada)
            ));
        }
        assert!(matches!(
            conferir("<a id='lnkInfraUnidade'>A</a><a id='lnkInfraUnidade'>A</a>"),
            Err(FalhaLeitura::Controle(_))
        ));
    }

    // Diagnóstico da navegação na mesma fonte que já validou o leitor.
    // Não imprime URL, hash, HTML ou cookie da gravação.
    #[test]
    #[ignore = "ZIPs originais no IES_ACERVO_CACHE; offline"]
    fn cassette_menu_catalogo_unb_pen1() {
        use crate::cassette::acervo::gravacao;
        use crate::cassette::{
            ZipDoAcervo, archive_filename, conferir_cassette_ate, html_da_resposta,
        };
        let cache = std::env::var_os("IES_ACERVO_CACHE").map_or_else(
            || std::path::PathBuf::from(std::env::var("HOME").unwrap()).join(".cache/ies"),
            std::path::PathBuf::from,
        );
        // Inventário dos cards: Controle, catálogo e troca de unidade.
        for (caminho, esperadas) in [
            (
                "cassettes/unb/marcador-listar-estados-unb--c47ad7e28f89.md",
                [12, 3, 1],
            ),
            (
                "cassettes/pen1/marcador-listar-estados-pen1--8358611b6263.md",
                [1, 3, 1],
            ),
        ] {
            let card = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(caminho);
            let zip = ZipDoAcervo::ler(&cache.join(archive_filename(&card).unwrap())).unwrap();
            let captura = conferir_cassette_ate(&card, &zip, 120_000_000).unwrap();
            let instalacao = gravacao::instalacao(&captura.json).unwrap();
            let mut controles = 0;
            let mut catalogos = 0;
            let mut anterior = None;
            let mut inventario = [0, 0, 0];
            for r in gravacao::respostas(&captura.json) {
                // Só o Controle imediatamente anterior é candidato. Qualquer
                // interação intermediária, inclusive troca, rompe a cadeia.
                let candidato: Option<(String, String)> = anterior.take();
                assert_eq!(r.metodo, "GET", "{caminho}: passo {}", r.passo);
                match r.operacao {
                    "procedimento_controlar" => inventario[0] += 1,
                    "marcador_listar" => inventario[1] += 1,
                    "infra_trocar_unidade" => inventario[2] += 1,
                    _ => panic!("{caminho}: operação não inventariada no passo {}", r.passo),
                }
                if r.status != 200 {
                    if r.operacao == "marcador_listar" {
                        println!("{caminho}: passo {} unidade=NAO_200", r.passo);
                    }
                    continue;
                }
                if r.operacao == "procedimento_controlar" {
                    let html = HtmlServido::de_teste(&html_da_resposta(r.interacao).unwrap());
                    let servida = gravacao::url_sem_consulta(&captura.json, r.interacao);
                    let resultado = link_do_menu(&html, &servida, &instalacao);
                    assert!(
                        resultado.is_ok(),
                        "{caminho}: passo {} sem link utilizável",
                        r.passo
                    );
                    let unidade = crate::controle_contexto::unidade_html(&arvore(&html));
                    if let Ok(Some(unidade)) = unidade {
                        anterior = Some((unidade, resultado.unwrap()));
                    }
                    controles += 1;
                } else if r.operacao == "marcador_listar" {
                    let html = HtmlServido::de_teste(&html_da_resposta(r.interacao).unwrap());
                    let dom = arvore(&html);
                    let encadeada = candidato.as_ref().and_then(|(unidade, link)| {
                        (r.interacao["response"]["url"].as_str() == Some(link.as_str()))
                            .then_some(unidade.as_str())
                    });
                    let classe = match crate::controle_contexto::unidade_html(&dom) {
                        Ok(Some(unidade)) if encadeada.is_some_and(|u| u != unidade) => "CONFLITO",
                        Ok(Some(_)) => "DIRETA",
                        Ok(None) if dom.com_id("lnkInfraUnidade").next().is_some() => "INVALIDA",
                        Ok(None) if encadeada.is_some() => "LINK_EXATO",
                        Ok(None) => "SEM_VINCULO",
                        Err(_) => "INVALIDA",
                    };
                    // Nunca publicar a unidade, URL, hash ou corpo original.
                    println!("{caminho}: passo {} unidade={classe}", r.passo);
                    catalogos += 1;
                }
            }
            assert_eq!(inventario, esperadas, "{caminho}: inventário do card");
            assert!(controles > 0, "{caminho}: nenhum Controle original");
            assert!(catalogos > 0, "{caminho}: nenhum catálogo GET 200 original");
        }
    }

    #[test]
    fn forma_publica_da_cli() {
        for args in [
            vec!["ies", "marcador"],
            vec!["ies", "marcador", "listar"],
            vec!["ies", "--json", "marcador", "listar", "--inativos"],
        ] {
            assert!(crate::Cli::try_parse_from(args).is_ok());
        }
        assert!(crate::Cli::try_parse_from(["ies", "marcador", "listar", "--confirmar"]).is_err());
    }
}
