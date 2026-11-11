//! CLI Web de blocos: GET Controle, GET lista e, para bloco N, GET detalhe.
//! Paginação interna com um POST de formulário observado, sem JS, retry ou restauração.
use crate::acao_do_sei::AcaoDoSei;
use crate::bloco::{self, ErroBloco, TipoBloco};
use crate::bloco_detalhe::{self};
use crate::configuracao;
use crate::controle_reconhecido;
use crate::erro_web::ErroCaixa;
use crate::formulario_web::ErroSubmissaoFormulario;
use crate::pagina_do_controle::codificacao_da_tela;
use crate::transporte::envio_post::ler_tela_via;
use crate::transporte::sessao::ReferenciasDaSessao;
use crate::transporte::{self, FalhaTransporte};
use crate::visao_do_controle::{EnvioDoControle, EnvioHttps};
use crate::{Bloco, BlocoAcao, Cli};
use serde_json::{Value, json};
mod paginacao;
mod resolucao;
pub(crate) use resolucao::ler_bloco_por_numero_via;

#[derive(Debug, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum FalhaLeitura {
    /// O tokenizador de atributos recusou a tag.
    Atributo(#[source] crate::html_atributos::ErroAtributoHtml),
    Transporte(#[from] FalhaTransporte),
    Controle(#[from] ErroCaixa),
    Bloco(#[from] ErroBloco),
    Detalhe(#[source] crate::bloco_detalhe::ErroBlocoDetalhe),
    Submissao(#[from] ErroSubmissaoFormulario),
    UnidadeInesperada,
    HtmlDeBlocosInvalido,
    IdentidadeDoBlocoAmbigua,
    ListaParcialNaoExcluiColisao,
}
/// Lista ou detalhe de bloco, já tipados.
pub(crate) enum Leitura {
    Lista(crate::bloco::ListaDeBlocos),
    Detalhe(bloco_detalhe::BlocoDetalhado),
}

/// O que a leitura de blocos observou.
pub(crate) struct Lido {
    pub(crate) unidade: String,
    pub(crate) requisicoes: usize,
    pub(crate) leitura: Leitura,
}

/// Transporte injetável, mesmo caminho da produção; cada chamada faz um GET.
fn ler_via<V: EnvioDoControle>(
    sessao: &ReferenciasDaSessao,
    unidade_esperada: Option<&str>,
    tipo: TipoBloco,
    numero: Option<&str>,
    pagina: Option<u32>,
    envio: &mut V,
) -> Result<Value, FalhaLeitura> {
    let Lido {
        unidade,
        requisicoes,
        leitura,
    } = ler_tipado_via(sessao, unidade_esperada, tipo, numero, pagina, envio)?;
    Ok(match leitura {
        Leitura::Detalhe(detalhe) => {
            json!({"unidade":unidade,"origem":"web","requisicoes":requisicoes,"bloco":detalhe})
        }
        Leitura::Lista(lista) => {
            json!({"unidade":unidade,"origem":"web","requisicoes":requisicoes,"lista":lista})
        }
    })
}

/// A mesma leitura, tipada: o `bloco incluir` reaproveita o detalhe.
pub(crate) fn ler_tipado_via<V: EnvioDoControle>(
    sessao: &ReferenciasDaSessao,
    unidade_esperada: Option<&str>,
    tipo: TipoBloco,
    numero: Option<&str>,
    pagina: Option<u32>,
    envio: &mut V,
) -> Result<Lido, FalhaLeitura> {
    let mut requisicoes = 0usize;
    let controle = ler_tela_via(
        sessao.url_do_controle(),
        sessao.cookie(),
        sessao.origem(),
        |url, cookie| {
            requisicoes += 1;
            envio.get(url, cookie)
        },
    )?;
    let unidade = controle_reconhecido::unidade_do_controle(&controle.html)?;
    if unidade_esperada
        .is_some_and(|v| v.split_whitespace().collect::<Vec<_>>().join(" ") != unidade)
    {
        return Err(FalhaLeitura::UnidadeInesperada);
    }
    let url_lista = bloco::link_do_menu(&controle.instalacao, &controle.url, &controle.html, tipo)?;
    let cookie = sessao.cookie().valor()?;
    requisicoes += 1;
    let resposta = envio.get(&url_lista, cookie)?;
    let codificacao = codificacao_da_tela(&resposta.mime);
    let html = transporte::decodificar_resposta(resposta)?;
    let mut lista = bloco::ler_lista(&html, tipo)?;
    if pagina == Some(2) && tipo == TipoBloco::Interno {
        lista = paginacao::ler_pagina_dois(
            &lista,
            &html,
            &url_lista,
            codificacao,
            &controle.instalacao,
            sessao,
            envio,
        )?;
        requisicoes += 1;
    }
    if pagina == Some(1)
        && lista
            .lista
            .faixa_observada
            .as_ref()
            .is_some_and(|f| f.inicio != 1)
    {
        return Err(FalhaLeitura::Bloco(
            ErroBloco::PaginaDeBlocosNaoImplementada,
        ));
    }
    if let Some(numero) = numero {
        let href = lista.href_do_bloco(numero)?;
        let url = bloco::resolver_link(
            &controle.instalacao,
            &url_lista,
            href,
            AcaoDoSei::RelBlocoProtocoloListar,
            Some(numero),
        )?;
        requisicoes += 1;
        let resposta = envio.get(&url, cookie)?;
        let html = transporte::decodificar_resposta(resposta)?;
        let estado = lista
            .lista
            .itens
            .iter()
            .find(|b| b.numero == numero)
            .ok_or(FalhaLeitura::Bloco(ErroBloco::BlocoNaoEncontradoNaPagina))?
            .estado
            .as_str();
        let detalhe = bloco_detalhe::ler_detalhe(&html, numero, tipo, estado)
            .map_err(crate::bloco_cli::FalhaLeitura::Detalhe)?;
        return Ok(Lido {
            unidade,
            requisicoes,
            leitura: Leitura::Detalhe(detalhe),
        });
    }
    Ok(Lido {
        unidade,
        requisicoes,
        leitura: Leitura::Lista(lista.lista),
    })
}
#[expect(
    clippy::indexing_slicing,
    reason = "serde_json::Value: leitura ausente dá Null e a escrita é em objeto json! local; projeção de saída a substituir em #733"
)]
fn executar_leitura(
    cli: &Cli,
    tipo: TipoBloco,
    numero: Option<&str>,
    pagina: Option<u32>,
) -> anyhow::Result<()> {
    if pagina.is_some_and(|p| p > 2) || (pagina == Some(2) && tipo != TipoBloco::Interno) {
        return Err(FalhaLeitura::Bloco(ErroBloco::PaginaDeBlocosNaoImplementada).into());
    }
    let arquivos = configuracao::arquivos_de_acesso()?;
    let sessao =
        ReferenciasDaSessao::ler(&arquivos.url_controle, &arquivos.origem, &arquivos.cookie)?;
    let valor = if let Some(numero) = numero {
        let lido =
            ler_bloco_por_numero_via(&sessao, cli.unidade.as_deref(), numero, &mut EnvioHttps)?;
        let Leitura::Detalhe(detalhe) = lido.leitura else {
            return Err(FalhaLeitura::HtmlDeBlocosInvalido.into());
        };
        json!({"unidade":lido.unidade,"origem":"web","requisicoes":lido.requisicoes,"bloco":detalhe})
    } else {
        ler_via(
            &sessao,
            cli.unidade.as_deref(),
            tipo,
            numero,
            pagina,
            &mut EnvioHttps,
        )?
    };
    if cli.json {
        println!("{}", serde_json::to_string_pretty(&valor)?);
    } else if let Some(bloco) = valor.get("bloco") {
        println!(
            "IES: bloco {} ({}) — {} item(ns) observados; {} GET(s).",
            bloco["numero"],
            bloco["tipo"].as_str().unwrap_or("não identificado"),
            bloco["itens"].as_array().map_or(0, Vec::len),
            valor["requisicoes"]
        );
    } else {
        println!(
            "IES: blocos {}: {} na página observada de {} declarados; {} requisição(ões).",
            tipo.rotulo(),
            valor["lista"]["itens"].as_array().map_or(0, Vec::len),
            valor["lista"]["total_declarado"],
            valor["requisicoes"]
        );
        if let Some(itens) = valor["lista"]["itens"].as_array() {
            for item in itens {
                println!(
                    "{} | {} | {}",
                    item["numero"], item["estado"], item["descricao"]
                );
            }
        }
    }
    Ok(())
}
pub(crate) fn executar(cli: &Cli, bloco: &Bloco) -> anyhow::Result<()> {
    match (&bloco.numero, &bloco.acao) {
        (None, Some(BlocoAcao::Listar { tipo, pagina })) => {
            executar_leitura(cli, *tipo, None, *pagina)
        }
        (
            None,
            Some(BlocoAcao::Incluir {
                bloco,
                documento,
                processo,
                confirmar,
            }),
        ) => crate::bloco_incluir_cli::executar(
            cli,
            &bloco.to_string(),
            documento.as_ref(),
            processo.as_deref(),
            *confirmar,
        ),
        (Some(numero), None) => {
            executar_leitura(cli, TipoBloco::Assinatura, Some(&numero.to_string()), None)
        }
        (None, None) => {
            crate::esquema_cli::show_noun_help("bloco");
            Ok(())
        }
        (Some(_), Some(_)) => Err(UsoDoBloco::NumeroComSubcomando.into()),
    }
}

/// Combinação de argumentos que o clap não recusa sozinho.
#[derive(Debug, thiserror::Error)]
#[error("{self:?}")]
enum UsoDoBloco {
    NumeroComSubcomando,
}
#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;
    #[test]
    fn cli_aceita_listar_tipo_e_pagina_e_numero() {
        assert!(
            Cli::try_parse_from([
                "ies", "bloco", "listar", "--tipo", "interno", "--pagina", "1"
            ])
            .is_ok()
        );
        assert!(Cli::try_parse_from(["ies", "bloco", "2195"]).is_ok());
        assert!(Cli::try_parse_from(["ies", "bloco", "listar", "--tipo", "desconhecido"]).is_err());
        assert!(Cli::try_parse_from(["ies", "bloco", "listar", "--pagina", "0"]).is_err());
    }
}

#[cfg(test)]
mod tests_fluxo {
    use super::*;
    use crate::formulario_web::{RespostaDoPost, SubmissaoDerivada};
    use crate::transporte::RespostaWeb;
    use std::collections::VecDeque;
    use std::io::Cursor;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    const HASH: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const ORIGEM: &str = "https://sei.exemplo.gov.br/sei";
    struct Arquivos {
        pasta: PathBuf,
        sessao: ReferenciasDaSessao,
    }
    impl Arquivos {
        fn criar() -> Self {
            let pasta = std::env::temp_dir().join(format!(
                "ies-bloco-teste-{}-{}",
                std::process::id(),
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
            std::fs::create_dir(&pasta).unwrap();
            let arquivos = [
                (
                    "controle",
                    format!(
                        "{ORIGEM}/controlador.php?acao=procedimento_controlar&infra_hash={HASH}"
                    ),
                ),
                ("origem", ORIGEM.to_owned()),
                ("cookie", "PHPSESSID=SINTETICO".to_owned()),
            ];
            for (nome, texto) in arquivos {
                let caminho = pasta.join(nome);
                std::fs::write(&caminho, texto).unwrap();
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    std::fs::set_permissions(&caminho, std::fs::Permissions::from_mode(0o600))
                        .unwrap();
                }
            }
            let sessao = ReferenciasDaSessao::ler(
                &pasta.join("controle"),
                &pasta.join("origem"),
                &pasta.join("cookie"),
            )
            .unwrap();
            Self { pasta, sessao }
        }
    }
    impl Drop for Arquivos {
        fn drop(&mut self) {
            std::fs::remove_dir_all(&self.pasta).unwrap();
        }
    }
    struct EnvioFalso {
        telas: VecDeque<(String, String)>,
        urls: Vec<String>,
    }
    impl EnvioDoControle for EnvioFalso {
        type Corpo = Cursor<Vec<u8>>;
        fn post(
            &mut self,
            _: SubmissaoDerivada,
            _: &str,
        ) -> Result<RespostaDoPost<Self::Corpo>, FalhaTransporte> {
            panic!("leitura de blocos nunca pode fazer POST")
        }
        fn get(
            &mut self,
            url: &str,
            cookie: &str,
        ) -> Result<RespostaWeb<Self::Corpo>, FalhaTransporte> {
            assert_eq!(cookie, "PHPSESSID=SINTETICO");
            let (esperada, html) = self.telas.pop_front().expect("GET não previsto");
            assert_eq!(url, esperada);
            self.urls.push(url.to_owned());
            Ok(RespostaWeb {
                status_code: 200,
                mime: "text/html; charset=utf-8".to_owned(),
                content_length: None,
                localizacao: None,
                corpo: Cursor::new(html.into_bytes()),
            })
        }
    }
    fn envio(detalhe: bool) -> EnvioFalso {
        let controle = format!(
            "<html><a id='lnkInfraUnidade'>PPI</a><input id='hdnTipoVisualizacao' value='R'>\
             <table id='tblProcessosRecebidos'><caption>0 registros:</caption></table>\
             <table id='tblProcessosGerados'><caption>0 registros:</caption></table>\
             <a href='controlador.php?acao=bloco_assinatura_listar&amp;infra_hash={HASH}'>Blocos</a></html>"
        );
        let lista = format!(
            "<table id='tblBlocos'><caption>Lista (1 registros):</caption>\
             <tr><th></th><th>Número</th><th>Estado</th><th>Descrição</th></tr>\
             <tr><td><input type='checkbox'></td><td><a href='controlador.php?acao=rel_bloco_protocolo_listar&amp;id_bloco=2195&amp;infra_hash={HASH}'>2195</a></td>\
             <td>Gerado</td><td>Treinamento</td></tr></table>"
        );
        let mut telas = VecDeque::from([
            (
                format!("{ORIGEM}/controlador.php?acao=procedimento_controlar&infra_hash={HASH}"),
                controle,
            ),
            (
                format!("{ORIGEM}/controlador.php?acao=bloco_assinatura_listar&infra_hash={HASH}"),
                lista,
            ),
        ]);
        if detalhe {
            let html = "<table id='tblProtocolosBlocos'><caption>Lista (1 registros):</caption>\
                <tr><th></th><th>Processo</th><th>Documento</th><th>Anotações</th></tr>\
                <tr><td><input type='checkbox'></td><td>NUP</td><td>001</td><td></td></tr></table>";
            telas.push_back((
                format!("{ORIGEM}/controlador.php?acao=rel_bloco_protocolo_listar&id_bloco=2195&infra_hash={HASH}"),
                html.to_owned(),
            ));
        }
        EnvioFalso {
            telas,
            urls: Vec::new(),
        }
    }
    #[test]
    fn lista_sao_exatamente_dois_get_sem_post() {
        // Caso de Todos tests/paginas_reais/menu_blocos.html @323eb01.
        let arquivos = Arquivos::criar();
        let mut envio = envio(false);
        let valor = ler_via(
            &arquivos.sessao,
            Some("PPI"),
            TipoBloco::Assinatura,
            None,
            None,
            &mut envio,
        )
        .unwrap();
        assert_eq!(envio.urls.len(), 2);
        assert!(envio.telas.is_empty());
        assert_eq!(valor["requisicoes"], 2);
        assert_eq!(valor["lista"]["itens"][0]["quantidade_itens"], Value::Null);
        assert!(!valor.to_string().contains(HASH));
        assert!(!valor.to_string().contains("PHPSESSID"));
    }
    #[test]
    fn detalhe_sao_tres_get_sem_post_e_so_numero_da_linha() {
        // Caso de Todos tests/paginas_reais/bloco_com_documentos.html @323eb01.
        let arquivos = Arquivos::criar();
        let mut envio = envio(true);
        let valor = ler_via(
            &arquivos.sessao,
            Some("PPI"),
            TipoBloco::Assinatura,
            Some("2195"),
            None,
            &mut envio,
        )
        .unwrap();
        assert_eq!(envio.urls.len(), 3);
        assert!(envio.telas.is_empty());
        assert_eq!(valor["requisicoes"], 3);
        assert_eq!(valor["bloco"]["itens"][0]["documento"], "001");
    }
    #[test]
    fn unidade_diferente_interrompe_antes_do_link_de_bloco() {
        let arquivos = Arquivos::criar();
        let mut envio = envio(false);
        let erro = ler_via(
            &arquivos.sessao,
            Some("ERRADA"),
            TipoBloco::Assinatura,
            None,
            None,
            &mut envio,
        )
        .err()
        .unwrap();
        assert!(matches!(erro, FalhaLeitura::UnidadeInesperada));
        assert_eq!(envio.urls.len(), 1);
    }
}
