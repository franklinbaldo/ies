//! Travessia da consulta de acervo com o envio nativo, sem retry de POST.
use super::{
    ErroAcervo, ProcessoDoAcervo, conferir_pagina, detalhe, formulario_do_acervo,
    total_do_resultado,
};
use crate::acao_do_sei::AcaoDoSei;
use crate::formulario_web::{
    ConferenciaDaUnidade, EfeitoAvaliado, EnvioDoFormulario, ErroSubmissaoFormulario,
    EstadoDeSessao, OperacaoDeFormulario, ResultadoDoEnvio, SubmissaoDerivada, TelaServida,
    derivar_submissao,
};
use crate::instalacao::Instalacao;
use crate::pagina_do_controle::codificacao_da_tela;
use crate::tela_html::{HtmlServido, arvore};
use crate::transporte::envio_post::ler_tela_via;
use crate::transporte::sessao::ReferenciasDaSessao;
use crate::unidade_retornos::DataCivil;
use crate::visao_do_controle::{EnvioContado, EnvioDoControle, EnvioHttps, TelaAtual};

/// Reutiliza o formulário da página atual, sem fabricar action ou hash.
/// O índice é zero-based; o corpo original não foi persistido pelo gravador.
pub(super) fn formulario_da_pagina(
    instalacao: &Instalacao,
    tela: &TelaServida<'_>,
    unidade: &str,
    pagina: usize,
) -> Result<SubmissaoDerivada, ErroSubmissaoFormulario> {
    const PAGINAR: OperacaoDeFormulario = OperacaoDeFormulario {
        acao: crate::acao_do_sei::AcaoDoSei::EstatisticasDetalharUnidade,
        formulario: "frmEstatisticasLista",
        envio: EnvioDoFormulario::SubmitPorScript,
        campos_ocultos_exigidos: &["hdnInfraPaginaAtual"],
        campos_ocultos_reescritos: &["hdnInfraPaginaAtual"],
        campos_usuario: &[],
        resultado: ResultadoDoEnvio::ConferidoPorLeituraPosterior,
        unidade: ConferenciaDaUnidade::NaTelaDeOrigemDoLink,
        efeito: EfeitoAvaliado::AlteraEstadoDaSessao {
            o_que: EstadoDeSessao::PaginaOuFiltroLembrado,
        },
    };
    derivar_submissao(
        &PAGINAR,
        instalacao,
        tela,
        Some(unidade),
        &[("hdnInfraPaginaAtual", &pagina.to_string())],
    )
}

#[derive(Debug, thiserror::Error)]
#[error("{self:?}")]
pub(super) enum ErroEnvioPagina {
    Submissao(#[from] ErroSubmissaoFormulario),
    Redirecionamento,
}

/// Conserva a URL do POST e o MIME da resposta para a próxima derivação.
/// Reutiliza todas as verificações do envio comum, sem repetir a requisição.
pub(super) fn enviar_pagina_via<R: std::io::Read>(
    submissao: SubmissaoDerivada,
    cookie: &crate::transporte::sessao::CookieDaSessao,
    enviar: impl FnOnce(
        SubmissaoDerivada,
        &str,
    ) -> Result<
        crate::formulario_web::RespostaDoPost<R>,
        crate::transporte::FalhaTransporte,
    >,
) -> Result<(String, String, HtmlServido), ErroEnvioPagina> {
    use crate::formulario_web::{RespostaDoEnvio, enviar_submissao_via};
    let url = submissao.url().to_owned();
    let mut mime = String::new();
    let resposta = enviar_submissao_via(submissao, cookie, |post, cookie| {
        let resposta = enviar(post, cookie)?;
        mime.clone_from(&resposta.resposta.mime);
        Ok(resposta)
    })?;
    match resposta {
        RespostaDoEnvio::Html(html) => Ok((url, mime, html)),
        RespostaDoEnvio::Redirecionamento(_) => Err(ErroEnvioPagina::Redirecionamento),
    }
}

fn link_do_menu(instalacao: &Instalacao, html: &HtmlServido) -> Result<String, ErroAcervo> {
    use crate::arvore_processo::endereco::conferir_endereco_relativo_por;
    use crate::consulta_do_controlador::menciona_acao;
    let acao = AcaoDoSei::GerarEstatisticasUnidade;
    let dom = arvore(html);
    let mut links = dom
        .elementos("a")
        .filter_map(|a| a.atributo("href"))
        .filter(|href| menciona_acao(href, acao));
    let href = links
        .next()
        .ok_or(ErroAcervo::MenuDeEstatisticasNaoReconhecido)?;
    if links.next().is_some() {
        return Err(ErroAcervo::MenuDeEstatisticasNaoReconhecido);
    }
    conferir_endereco_relativo_por(instalacao, href, acao, "acao", acao.como_na_url())
        .map_err(|_| ErroAcervo::MenuDeEstatisticasNaoReconhecido)
}

fn tela_servida(tela: &TelaAtual) -> Result<TelaServida<'_>, ErroAcervo> {
    Ok(TelaServida {
        url: &tela.url,
        html: &tela.html,
        codificacao: codificacao_da_tela(&tela.mime)
            .ok_or(ErroAcervo::CharsetDaTelaNaoSuportado)?,
    })
}

fn get<V: EnvioDoControle>(
    envio: &mut V,
    sessao: &ReferenciasDaSessao,
    url: String,
) -> Result<TelaAtual, crate::transporte::FalhaTransporte> {
    // O chamador fornece somente o link conferido na mesma instalação.
    let resposta = envio.get(&url, sessao.cookie().valor()?)?;
    let mime = resposta.mime.clone();
    let html = crate::transporte::decodificar_resposta(resposta)?;
    Ok(TelaAtual { url, html, mime })
}

fn post<V: EnvioDoControle>(
    envio: &mut V,
    sessao: &ReferenciasDaSessao,
    submissao: SubmissaoDerivada,
) -> Result<TelaAtual, ErroEnvioPagina> {
    let (url, mime, html) = enviar_pagina_via(submissao, sessao.cookie(), |post, cookie| {
        envio.post(post, cookie)
    })?;
    Ok(TelaAtual { url, html, mime })
}

fn ler_via<V: EnvioDoControle>(
    sessao: &ReferenciasDaSessao,
    unidade_esperada: Option<&str>,
    data: DataCivil,
    envio: &mut V,
) -> anyhow::Result<(String, Vec<ProcessoDoAcervo>)> {
    let controle = ler_tela_via(
        sessao.url_do_controle(),
        sessao.cookie(),
        sessao.origem(),
        |url, cookie| envio.get(url, cookie),
    )?;
    let unidade = crate::controle_reconhecido::unidade_do_controle(&controle.html)?;
    if unidade_esperada.is_some_and(|esperada| esperada != unidade) {
        return Err(ErroAcervo::UnidadeDiferenteDaSolicitada.into());
    }
    let inst = &controle.instalacao;
    let url = link_do_menu(inst, &controle.html)?;
    let tela = get(envio, sessao, url)?;
    let consulta = formulario_do_acervo(inst, &tela_servida(&tela)?, &unidade, data)?;
    let resultado = post(envio, sessao, consulta)?;
    let (total, url) = total_do_resultado(&resultado.html, inst)?;
    let Some(url) = url else {
        return Ok((unidade, Vec::new()));
    };
    let mut tela = get(envio, sessao, url)?;
    let mut processos = Vec::new();
    let mut indice = 0;
    loop {
        let pagina = detalhe(&tela.html)?;
        conferir_pagina(&pagina, total, processos.len(), indice == 0)?;
        processos.extend(pagina.processos);
        if processos.len() == total {
            return Ok((unidade, processos));
        }
        indice += 1;
        let submissao = formulario_da_pagina(inst, &tela_servida(&tela)?, &unidade, indice)?;
        tela = post(envio, sessao, submissao)?;
    }
}

pub(crate) fn listar(
    unidade: Option<&str>,
    json_mode: bool,
    data: DataCivil,
) -> anyhow::Result<()> {
    let arquivos = crate::configuracao::arquivos_de_acesso()?;
    let sessao =
        ReferenciasDaSessao::ler(&arquivos.url_controle, &arquivos.origem, &arquivos.cookie)?;
    let mut envio = EnvioContado::novo(EnvioHttps);
    // Não envolver em relogin que recomeçaria os POSTs de paginação.
    let (unidade, processos) = ler_via(&sessao, unidade, data, &mut envio)?;
    let total = processos.len();
    let distintos = processos
        .iter()
        .map(|p| p.processo.as_str())
        .collect::<std::collections::HashSet<_>>()
        .len();
    if json_mode {
        println!(
            "{}",
            serde_json::json!({
                "unidade": unidade, "data": data.to_string(), "origem": "web",
                "total_declarado": total, "linhas_observadas": total,
                "nups_distintos": distintos, "processos": processos,
                "requisicoes": envio.pedidos.total(),
            })
        );
    } else {
        println!(
            "Acervo da unidade {unidade} em {data}: {total} linhas, {distintos} NUPs distintos."
        );
        for processo in processos {
            println!(
                "{} | {} | {} | {} | {} | {}",
                processo.processo,
                processo.tipo,
                processo.orgao,
                processo.unidade,
                processo.mes,
                processo.ano
            );
        }
    }
    Ok(())
}

#[test]
fn menu_exige_link_unico_assinado_da_instalacao() {
    use crate::formulario_web::tests::{HASH_TELA, instalacao};
    let inst = instalacao();
    let link = format!(
        "<a href='controlador.php?acao=gerar_estatisticas_unidade&amp;infra_hash={HASH_TELA}'>Estatísticas</a>"
    );
    assert!(link_do_menu(&inst, &HtmlServido::de_teste(&link)).is_ok());
    for html in [
        String::new(),
        format!("{link}{link}"),
        link.replace("controlador.php?", "https://fora.example/controlador.php?"),
        link.replace(HASH_TELA, "x"),
    ] {
        assert!(link_do_menu(&inst, &HtmlServido::de_teste(&html)).is_err());
    }
}

#[cfg(test)]
mod replay {
    use super::*;
    use crate::cassette::acervo::gravacao as fonte;
    use crate::transporte::testes_http::resposta_post;
    use crate::transporte::{FalhaTransporte, RespostaWeb};
    use std::collections::VecDeque;
    use std::io::Cursor;
    use std::path::{Path, PathBuf};

    // Os bytes, status e MIME são originais; a query da requisição e o
    // corpo POST não foram gravados. A conferência é de método/ação/rota,
    // não de igualdade de requisições nem da data histórica consultada.
    struct Gravado<'a> {
        json: &'a serde_json::Value,
        passos: VecDeque<fonte::Resposta<'a>>,
        pagina: usize,
    }

    impl Gravado<'_> {
        fn proxima(
            &mut self,
            metodo: &str,
            url: &str,
        ) -> crate::formulario_web::RespostaDoPost<Cursor<Vec<u8>>> {
            let r = self.passos.pop_front().expect("requisição além do roteiro");
            assert_eq!(metodo, r.metodo, "passo {}", r.passo);
            let (rota, query) = url.split_once('?').expect("consulta da requisição");
            assert!(
                rota == fonte::url_sem_consulta(self.json, r.interacao),
                "rota divergente no passo {}",
                r.passo
            );
            assert!(
                query
                    .split('&')
                    .any(|p| p.strip_prefix("acao=") == Some(r.operacao)),
                "ação divergente no passo {}",
                r.passo
            );
            resposta_post(
                i32::try_from(r.status).unwrap(),
                &fonte::mime(r.interacao),
                fonte::corpo(r.interacao),
                None,
                fonte::localizacao(r.interacao).map(str::to_owned),
            )
        }
    }

    impl EnvioDoControle for Gravado<'_> {
        type Corpo = Cursor<Vec<u8>>;

        fn post(
            &mut self,
            submissao: SubmissaoDerivada,
            cookie: &str,
        ) -> Result<crate::formulario_web::RespostaDoPost<Self::Corpo>, FalhaTransporte> {
            assert!(cookie == "PHPSESSID=SINTETICO");
            // O corpo gravado não existe; conferir a intenção da CLI contra
            // um oráculo independente, sem inferir igualdade com o POST real.
            let corpo = std::str::from_utf8(submissao.corpo()).expect("corpo URL encoded");
            let conferir = |nome: &str, esperado: &str| {
                let mut valores = corpo
                    .split('&')
                    .filter_map(|par| par.split_once('='))
                    .filter(|(chave, _)| *chave == nome)
                    .map(|(_, valor)| valor);
                assert!(
                    valores.next() == Some(esperado) && valores.next().is_none(),
                    "campo {nome} divergente ou repetido"
                );
            };
            match self.passos.front().expect("POST além do roteiro").operacao {
                "gerar_estatisticas_unidade" => {
                    conferir("selOrgao", "");
                    conferir("txtPeriodoDe", "29%2F02%2F2024");
                    conferir("txtPeriodoA", "29%2F02%2F2024");
                    conferir("sbmPesquisar", "Pesquisar");
                }
                "estatisticas_detalhar_unidade" => {
                    self.pagina += 1;
                    conferir("hdnInfraPaginaAtual", &self.pagina.to_string());
                }
                _ => panic!("POST inesperado no roteiro"),
            }
            Ok(self.proxima("POST", submissao.url()))
        }

        fn get(
            &mut self,
            url: &str,
            cookie: &str,
        ) -> Result<RespostaWeb<Self::Corpo>, FalhaTransporte> {
            assert!(cookie == "PHPSESSID=SINTETICO");
            Ok(self.proxima("GET", url).resposta)
        }
    }

    fn roteiro(json: &serde_json::Value, paginas: usize) -> Gravado<'_> {
        let respostas = fonte::respostas(json);
        let detalhe = respostas
            .iter()
            .position(|r| r.operacao == "estatisticas_detalhar_unidade")
            .expect("detalhe original");
        // A gravação contém duas consultas estatísticas; só a última
        // precede a travessia do detalhe. Não simular a consulta anterior.
        let relatorio = respostas[..detalhe]
            .iter()
            .rposition(|r| r.operacao == "gerar_estatisticas_unidade" && r.metodo == "POST")
            .expect("relatório original");
        let formulario = respostas[..relatorio]
            .iter()
            .rposition(|r| r.operacao == "gerar_estatisticas_unidade" && r.metodo == "GET")
            .expect("formulário original");
        let controle = respostas[..formulario]
            .iter()
            .rposition(|r| r.operacao == "procedimento_controlar" && r.metodo == "GET")
            .expect("Controle original");
        let mut passos = VecDeque::new();
        for r in respostas {
            if [controle, formulario, relatorio].contains(&r.passo)
                || (detalhe..detalhe + paginas).contains(&r.passo)
            {
                assert_eq!(r.status, 200, "passo {}", r.passo);
                passos.push_back(r);
            }
        }
        assert_eq!(passos.len(), paginas + 3);
        Gravado {
            json,
            passos,
            pagina: 0,
        }
    }

    #[test]
    #[ignore = "acervo original: exige ZIPs verificados no cache"]
    fn acervo_detalhe_original_fluxo() {
        use crate::arquivo_privado::{Politica, gravar};
        use crate::cassette::{ZipDoAcervo, conferir_cassette_ate};

        let raiz = Path::new(env!("CARGO_MANIFEST_DIR")).join("cassettes");
        let cache = std::env::var_os("IES_ACERVO_CACHE").map_or_else(
            || PathBuf::from(std::env::var_os("HOME").expect("HOME")).join(".cache/ies"),
            PathBuf::from,
        );
        for (nome, revisao, total, paginas) in [
            ("unb", "74b641c4a8a1", 21, 1),
            ("pen1", "5c557e93a6ce", 305, 7),
            ("pen2", "c64c576f9a9a", 685, 14),
        ] {
            let card = raiz
                .join(nome)
                .join(format!("unidade-estatisticas-acervo-{nome}--{revisao}.md"));
            let arquivo = crate::cassette::archive_filename(&card).expect("card OKF");
            let zip = ZipDoAcervo::ler(&cache.join(&arquivo)).expect("ZIP original");
            let conferida =
                conferir_cassette_ate(&card, &zip, 120_000_000).expect("integridade do original");
            assert_eq!(conferida.instalacao, nome);
            let inst = fonte::instalacao(&conferida.json).expect("instalação original");
            let mut envio = EnvioContado::novo(roteiro(&conferida.json, paginas));

            // URL inicial e cookie sintéticos, nunca enviados à rede.
            let origem = inst.url_base();
            let url = format!(
                "{origem}/controlador.php?acao=procedimento_controlar&infra_hash={}",
                "a".repeat(64)
            );
            let pasta = std::env::temp_dir().join(format!(
                "ies-acervo-replay-{}-{nome}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
            std::fs::create_dir(&pasta).unwrap();
            for (arquivo, valor) in [
                ("url", url.as_str()),
                ("origem", origem.as_str()),
                ("cookie", "PHPSESSID=SINTETICO"),
            ] {
                gravar(
                    &pasta.join(arquivo),
                    valor.as_bytes(),
                    Politica::exclusiva().diretorio_do_ies(),
                )
                .unwrap();
            }
            let sessao = ReferenciasDaSessao::ler(
                &pasta.join("url"),
                &pasta.join("origem"),
                &pasta.join("cookie"),
            )
            .unwrap();
            let data = crate::cli_unidade_aux::data_do_acervo("2024-02-29").unwrap();
            let resultado = ler_via(&sessao, None, data, &mut envio);
            std::fs::remove_dir_all(pasta).unwrap();

            let (_, processos) = resultado.unwrap_or_else(|_| panic!("{nome}: fluxo recusado"));
            assert!(envio.interno.passos.is_empty(), "{nome}: roteiro restante");
            assert_eq!(envio.total(), paginas + 3, "{nome}");
            assert_eq!(processos.len(), total, "{nome}");
            let distintos = processos
                .iter()
                .map(|p| p.processo.as_str())
                .collect::<std::collections::HashSet<_>>()
                .len();
            assert_eq!(distintos, total, "{nome}");
            println!("{nome}: fluxo={total} requisições={}", envio.total());
        }
    }
}
