//! CLI Web de blocos: GET Controle, GET lista e, para bloco N, GET detalhe.
//! Zero POST, JS, retry ou restauração da tela.
use crate::bloco::{self, ErroBloco, TipoBloco};
use crate::bloco_detalhe::{self, ErroDetalheBloco};
use crate::configuracao;
use crate::controle_reconhecido;
use crate::erro_web::ErroCaixa;
use crate::transporte::envio_post::ler_tela_via;
use crate::transporte::sessao::ReferenciasDaSessao;
use crate::transporte::{self, FalhaTransporte};
use crate::visao_do_controle::{EnvioDoControle, EnvioHttps};
use crate::{Bloco, BlocoAcao, Cli, erro_fonte};
use serde_json::{Value, json};
use std::process::ExitCode;

#[derive(Debug, crate::erro_publico::Causa)]
enum FalhaLeitura {
    #[causa(interna)]
    Transporte(FalhaTransporte),
    #[causa(interna)]
    Controle(ErroCaixa),
    #[causa(nome = "unidade_inesperada_na_leitura_de_blocos")]
    UnidadeInesperada,
    #[causa(interna)]
    Bloco(ErroBloco),
    #[causa(interna)]
    Detalhe(ErroDetalheBloco),
}
impl FalhaLeitura {
    fn subcodigo(&self) -> &'static str {
        crate::erro_publico::folha(self)
    }
    fn mensagem(&self) -> &'static str {
        match self {
            Self::Transporte(e) => e.mensagem(),
            Self::Controle(_) => "Controle não reconhecido na leitura de blocos",
            Self::UnidadeInesperada => "unidade atual diferente da solicitada",
            Self::Bloco(e) => e.mensagem(),
            Self::Detalhe(_) => "a tabela de documentos ou processos do bloco não foi reconhecida",
        }
    }
    fn status_http(&self) -> Option<&transporte::StatusHttpRecusado> {
        match self {
            Self::Transporte(e) => e.status_http(),
            _ => None,
        }
    }
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
    let html = transporte::decodificar_resposta(resposta)?;
    let lista = bloco::ler_lista(&html, tipo)?;
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
            "rel_bloco_protocolo_listar",
            Some(numero),
        )?;
        requisicoes += 1;
        let resposta = envio.get(&url, cookie)?;
        let html = transporte::decodificar_resposta(resposta)?;
        let detalhe = bloco_detalhe::ler_detalhe(&html, numero, tipo)?;
        return Ok(
            json!({"unidade":unidade,"origem":"web","requisicoes":requisicoes,"bloco":detalhe}),
        );
    }
    Ok(json!({"unidade":unidade,"origem":"web","requisicoes":requisicoes,"lista":lista.lista}))
}
fn executar_leitura(
    cli: &Cli,
    tipo: TipoBloco,
    numero: Option<&str>,
    pagina: Option<u32>,
) -> ExitCode {
    if let Some(numero) = numero
        && (numero.is_empty() || !numero.bytes().all(|b| b.is_ascii_digit()))
    {
        return erro_uso(
            cli.json,
            "numero_do_bloco_invalido",
            "número de bloco deve ser decimal",
        );
    }
    if pagina.is_some_and(|p| p != 1) {
        return erro_fonte(
            cli.json,
            ErroBloco::PaginaDeBlocosNaoImplementada.subcodigo(),
            "paginação de blocos ainda não implementada: nenhuma requisição enviada",
        );
    }
    let arquivos = match configuracao::arquivos_de_acesso() {
        Ok(a) => a,
        Err(e) => return erro_fonte(cli.json, e.subcodigo(), e.mensagem()),
    };
    let sessao = match ReferenciasDaSessao::ler(
        &arquivos.url_controle,
        &arquivos.origem,
        &arquivos.cookie,
    ) {
        Ok(s) => s,
        Err(e) => {
            return crate::saida_erro::erro_fonte_http(
                cli.json,
                e.subcodigo(),
                e.mensagem(),
                e.status_http(),
            );
        }
    };
    match ler_via(
        &sessao,
        cli.unidade.as_deref(),
        tipo,
        numero,
        pagina,
        &mut EnvioHttps,
    ) {
        Ok(valor) => {
            if cli.json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&valor).unwrap_or_else(|_| "{}".to_owned())
                );
            } else if let Some(bloco) = valor.get("bloco") {
                println!(
                    "IES: bloco {} ({}) — {} item(ns) observados; {} GET(s).",
                    bloco["numero"],
                    tipo.rotulo(),
                    bloco["itens"].as_array().map_or(0, Vec::len),
                    valor["requisicoes"]
                );
            } else {
                println!(
                    "IES: blocos {}: {} na página observada de {} declarados; {} GET(s).",
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
            ExitCode::SUCCESS
        }
        Err(e) => crate::saida_erro::erro_fonte_http(
            cli.json,
            e.subcodigo(),
            e.mensagem(),
            e.status_http(),
        ),
    }
}
fn erro_uso(json_mode: bool, subcodigo: &str, mensagem: &str) -> ExitCode {
    if json_mode {
        println!(
            "{}",
            json!({"codigo":"entrada_invalida","subcodigo":subcodigo,
            "erro":mensagem,"retentavel":false})
        );
    } else {
        eprintln!("IES: {mensagem} [{subcodigo}]");
    }
    ExitCode::from(2)
}
pub(crate) fn executar(cli: &Cli, bloco: &Bloco) -> ExitCode {
    match (&bloco.numero, &bloco.acao) {
        (None, Some(BlocoAcao::Listar { tipo, pagina })) => {
            executar_leitura(cli, *tipo, None, *pagina)
        }
        (Some(numero), None) => executar_leitura(cli, TipoBloco::Assinatura, Some(numero), None),
        (None, None) => {
            crate::esquema_cli::show_noun_help("bloco");
            ExitCode::SUCCESS
        }
        _ => erro_uso(
            cli.json,
            "opcoes_de_bloco_incompativeis",
            "número e listagem não podem ser combinados",
        ),
    }
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
