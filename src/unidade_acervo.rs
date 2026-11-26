//! Primeiro núcleo de leitura de acervo histórico (#878).
//! Forma de detalhe confrontada com `UnB`, PEN1 e PEN2 na CI 45745032222.
use crate::legenda::{ErroLegenda, Legenda, ler_legenda};
use crate::tela_html::{Elemento, HtmlServido, arvore};

#[derive(Debug, PartialEq, Eq)]
struct ProcessoDoAcervo {
    processo: String,
    tipo: String,
    orgao: String,
    unidade: String,
    mes: String,
    ano: String,
}

#[derive(Debug, PartialEq, Eq)]
struct Pagina {
    legenda: Legenda,
    processos: Vec<ProcessoDoAcervo>,
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
enum ErroAcervo {
    TelaDesconhecida,
    Legenda(#[from] ErroLegenda),
    FaixaIncompleta,
    ResultadoSemTotal,
}

fn texto(no: Elemento<'_>) -> String {
    no.texto().split_whitespace().collect::<Vec<_>>().join(" ")
}

const ANDAMENTO_ABERTO: &str = "Processos com andamento aberto na unidade ao final do período";

/// Seleciona o TOTAL da tabela 4, não um tipo ou outra estatística.
/// Resolve o link observado com a guarda comum dos endereços assinados.
fn total_do_resultado(
    html: &HtmlServido,
    instalacao: &crate::instalacao::Instalacao,
) -> Result<(usize, String), ErroAcervo> {
    use crate::acao_do_sei::AcaoDoSei;
    use crate::arvore_processo::endereco::conferir_endereco_relativo_por;
    use crate::consulta_do_controlador::ConsultaDoControlador;
    let dom = arvore(html);
    let mut tabelas = dom.elementos("table").filter(|t| {
        t.elementos("caption")
            .any(|c| texto(c).trim_end_matches(':') == ANDAMENTO_ABERTO)
    });
    let tabela = tabelas.next().ok_or(ErroAcervo::ResultadoSemTotal)?;
    if tabelas.next().is_some() {
        return Err(ErroAcervo::ResultadoSemTotal);
    }
    let mut linhas = tabela.elementos("tr").filter(|tr| {
        tr.atributo("class")
            .is_some_and(|c| c.split_whitespace().any(|v| v == "totalEstatisticas"))
    });
    let linha = linhas.next().ok_or(ErroAcervo::ResultadoSemTotal)?;
    if linhas.next().is_some() {
        return Err(ErroAcervo::ResultadoSemTotal);
    }
    let mut links = linha.elementos("a").filter_map(|a| {
        let (_, resto) = a.atributo("onclick")?.split_once("abrirDetalhe('")?;
        let (url, _) = resto.split_once('\'')?;
        Some((texto(a), url.replace("&amp;", "&")))
    });
    let (numero, url) = links.next().ok_or(ErroAcervo::ResultadoSemTotal)?;
    if links.next().is_some() {
        return Err(ErroAcervo::ResultadoSemTotal);
    }
    let (_, query) = url.split_once('?').ok_or(ErroAcervo::ResultadoSemTotal)?;
    let consulta = ConsultaDoControlador::ler(query).map_err(|_| ErroAcervo::ResultadoSemTotal)?;
    if consulta.valores("id_tipo_procedimento").next().is_some() {
        return Err(ErroAcervo::ResultadoSemTotal);
    }
    let url = conferir_endereco_relativo_por(
        instalacao,
        &url,
        AcaoDoSei::EstatisticasDetalharUnidade,
        "tabela_estatisticas",
        "4",
    )
    .map_err(|_| ErroAcervo::ResultadoSemTotal)?;
    let total = ler_legenda(&format!("{numero} registros:"))?.total;
    Ok((total, url))
}

// Hipótese de submissão baseada no Todos; não abre sessão nem envia POST.
fn formulario_do_acervo(
    instalacao: &crate::instalacao::Instalacao,
    tela: &crate::formulario_web::TelaServida<'_>,
    unidade: &str,
    data: crate::unidade_retornos::DataCivil,
) -> Result<crate::formulario_web::SubmissaoDerivada, crate::formulario_web::ErroSubmissaoFormulario>
{
    use crate::acao_do_sei::AcaoDoSei;
    use crate::formulario_web::{
        ConferenciaDaUnidade, EfeitoAvaliado, EnvioDoFormulario, OperacaoDeFormulario,
        ResultadoDoEnvio, derivar_submissao,
    };
    const CONSULTAR: OperacaoDeFormulario = OperacaoDeFormulario {
        acao: AcaoDoSei::GerarEstatisticasUnidade,
        formulario: "frmEstatisticas",
        envio: EnvioDoFormulario::BotaoDeEnvio("sbmPesquisar"),
        campos_ocultos_exigidos: &[],
        campos_ocultos_reescritos: &[],
        campos_usuario: &["selOrgao", "txtPeriodoDe", "txtPeriodoA"],
        resultado: ResultadoDoEnvio::ConferidoPorLeituraPosterior,
        unidade: ConferenciaDaUnidade::NaTelaDeOrigemDoLink,
        efeito: EfeitoAvaliado::ConsultaSemEfeitoObservado {
            evidencia: "Todos estatisticas.py: consulta historica, sem escrita observada",
        },
    };
    let data = data.to_string();
    derivar_submissao(
        &CONSULTAR,
        instalacao,
        tela,
        Some(unidade),
        &[
            ("selOrgao", ""),
            ("txtPeriodoDe", &data),
            ("txtPeriodoA", &data),
        ],
    )
}

#[test]
fn formulario_sintetico_envia_orgao_vazio_e_um_dia() {
    use crate::formulario_web::tests::{HASH_TELA, instalacao, tela_iso};
    let url = format!(
        "https://sei.exemplo.gov.br/sei/controlador.php?acao=gerar_estatisticas_unidade&infra_hash={HASH_TELA}"
    );
    let html = HtmlServido::de_teste(&format!(
        "<a id='lnkInfraUnidade'>U</a><form id='frmEstatisticas' method='post' action='controlador.php?acao=gerar_estatisticas_unidade&amp;infra_hash={HASH_TELA}'>\
         <select name='selOrgao'><option value='null' selected>Todos</option></select>\
         <input name='txtPeriodoDe'><input name='txtPeriodoA'>\
         <input type='submit' name='sbmPesquisar' value='Pesquisar'></form>"
    ));
    let data = crate::cli_unidade_aux::data_do_acervo("2024-02-29").unwrap();
    let post = formulario_do_acervo(&instalacao(), &tela_iso(&url, &html), "U", data).unwrap();
    assert_eq!(
        post.nomes_dos_campos(),
        ["selOrgao", "txtPeriodoDe", "txtPeriodoA", "sbmPesquisar"]
    );
    assert_eq!(
        std::str::from_utf8(post.corpo()).unwrap(),
        "selOrgao=&txtPeriodoDe=29%2F02%2F2024&txtPeriodoA=29%2F02%2F2024&sbmPesquisar=Pesquisar"
    );
    let ausente = HtmlServido::de_teste(&html.as_str().replace("txtPeriodoA", "outro"));
    assert!(formulario_do_acervo(&instalacao(), &tela_iso(&url, &ausente), "U", data).is_err());
    assert_eq!(
        formulario_do_acervo(&instalacao(), &tela_iso(&url, &html), "OUTRA", data).unwrap_err(),
        crate::formulario_web::ErroSubmissaoFormulario::UnidadeDaTelaDiferenteDaSolicitada
    );
}

#[test]
fn data_iso_reusa_calendario_civil() {
    use crate::cli_unidade_aux::data_do_acervo;
    assert_eq!(
        data_do_acervo("2024-02-29").unwrap().to_string(),
        "29/02/2024"
    );
    for s in [
        "2026-02-29",
        "2026-04-31",
        "2026-13-01",
        "2026-11-00",
        "20/11/2026",
        "2026-1-01",
        "2026-01-1",
        "0000-01-01",
    ] {
        assert!(data_do_acervo(s).is_err(), "{s}");
    }
}

#[path = "unidade_acervo/paginacao_tests.rs"]
mod paginacao_tests;
#[path = "unidade_acervo/total_tests.rs"]
mod total_tests;

fn detalhe(html: &HtmlServido) -> Result<Pagina, ErroAcervo> {
    let dom = arvore(html);
    let tabelas: Vec<_> = dom
        .elementos("table")
        .filter(|t| {
            t.elementos("caption")
                .any(|c| texto(c).starts_with("Lista de Registros de Detalhamento"))
        })
        .collect();
    let [tabela] = tabelas.as_slice() else {
        return Err(ErroAcervo::TelaDesconhecida);
    };
    let legendas: Vec<_> = tabela.elementos("caption").collect();
    let [legenda] = legendas.as_slice() else {
        return Err(ErroAcervo::TelaDesconhecida);
    };
    let legenda = ler_legenda(&texto(*legenda))?;
    let colunas: Vec<_> = tabela.elementos("th").map(texto).collect();
    let indice = |nome| {
        let mut posicoes = colunas
            .iter()
            .enumerate()
            .filter(|(_, valor)| valor.as_str() == nome);
        let (i, _) = posicoes.next().ok_or(ErroAcervo::TelaDesconhecida)?;
        if posicoes.next().is_some() {
            return Err(ErroAcervo::TelaDesconhecida);
        }
        Ok(i)
    };
    let [orgao, unidade, processo, tipo, mes, ano] = [
        indice("Órgão")?,
        indice("Unidade")?,
        indice("Processo")?,
        indice("Tipo")?,
        indice("Mês")?,
        indice("Ano")?,
    ];
    let mut processos = Vec::new();
    for linha in tabela.elementos("tr") {
        let celulas: Vec<_> = linha.filhos().filter(|c| c.nome() == "td").collect();
        if celulas.is_empty() {
            continue;
        }
        if celulas.len() != colunas.len() {
            return Err(ErroAcervo::TelaDesconhecida);
        }
        let nup = texto(celulas[processo]);
        if nup.is_empty() {
            return Err(ErroAcervo::TelaDesconhecida);
        }
        processos.push(ProcessoDoAcervo {
            processo: nup,
            tipo: texto(celulas[tipo]),
            orgao: texto(celulas[orgao]),
            unidade: texto(celulas[unidade]),
            mes: texto(celulas[mes]),
            ano: texto(celulas[ano]),
        });
    }
    let linhas = processos.len();
    let coerente = match &legenda.faixa {
        Some(f) => {
            f.inicio > 0
                && f.inicio <= f.fim
                && f.fim <= legenda.total
                && f.fim - f.inicio + 1 == linhas
        }
        None => legenda.total == linhas,
    };
    if !coerente {
        return Err(ErroAcervo::FaixaIncompleta);
    }
    Ok(Pagina { legenda, processos })
}

fn conferir_pagina(
    pagina: &Pagina,
    total: usize,
    anteriores: usize,
    primeira: bool,
) -> Result<(), ErroAcervo> {
    if pagina.legenda.total != total || (!primeira && anteriores == total) {
        return Err(ErroAcervo::FaixaIncompleta);
    }
    let linhas = pagina.processos.len();
    let coerente = match &pagina.legenda.faixa {
        Some(f) => {
            f.inicio.checked_sub(1) == Some(anteriores)
                && f.fim.checked_sub(anteriores) == Some(linhas)
                && f.fim <= total
                && linhas > 0
        }
        None => anteriores == 0 && linhas == total,
    };
    if !coerente {
        return Err(ErroAcervo::FaixaIncompleta);
    }
    Ok(())
}

/// Reúne a primeira travessia sem deduplicar as linhas.
fn reunir(paginas: impl IntoIterator<Item = Pagina>) -> Result<Vec<ProcessoDoAcervo>, ErroAcervo> {
    let mut paginas = paginas.into_iter();
    let primeira = paginas.next().ok_or(ErroAcervo::FaixaIncompleta)?;
    let total = primeira.legenda.total;
    let mut processos = Vec::new();
    for (i, pagina) in std::iter::once(primeira).chain(paginas).enumerate() {
        conferir_pagina(&pagina, total, processos.len(), i == 0)?;
        processos.extend(pagina.processos);
    }
    if processos.len() != total {
        return Err(ErroAcervo::FaixaIncompleta);
    }
    Ok(processos)
}

#[test]
fn detalhe_preserva_identidade_e_faixa() {
    let tela = HtmlServido::de_teste(
        "<table><caption>Lista de Registros de Detalhamento (3 registros - 1 a 2):</caption>\
         <tr><th></th><th>Órgão</th><th>Unidade</th><th>Processo</th><th>Tipo</th><th>Mês</th><th>Ano</th></tr>\
         <tr><td></td><td>O</td><td>U</td><td>0001.000001/2026-11</td><td>T</td><td>10</td><td>2024</td></tr>\
         <tr><td></td><td>O</td><td>U</td><td>0001.000001/2026-11</td><td>T</td><td>10</td><td>2024</td></tr></table>",
    );
    let lida = detalhe(&tela).unwrap();
    assert_eq!(lida.legenda.total, 3);
    assert_eq!(lida.processos.len(), 2);
    assert_eq!(lida.processos[0].processo, "0001.000001/2026-11");
    assert_eq!(lida.processos[0].ano, "2024");
    assert_eq!(lida.processos[0], lida.processos[1]);
    let faixa_errada = tela.as_str().replace("1 a 2", "1 a 3");
    assert_eq!(
        detalhe(&HtmlServido::de_teste(&faixa_errada)),
        Err(ErroAcervo::FaixaIncompleta)
    );
}

#[test]
fn tela_ausente_nao_e_zero() {
    assert_eq!(
        detalhe(&HtmlServido::de_teste("<p>Outra tela</p>")),
        Err(ErroAcervo::TelaDesconhecida)
    );
}

#[test]
fn travessia_exige_cobertura_sem_deduplicar() {
    use crate::legenda::Faixa;
    let pagina = |total, inicio, fim| Pagina {
        legenda: Legenda {
            total,
            faixa: Some(Faixa { inicio, fim }),
        },
        processos: (inicio..=fim)
            .map(|_| ProcessoDoAcervo {
                processo: "A".into(),
                tipo: "T".into(),
                orgao: "O".into(),
                unidade: "U".into(),
                mes: "10".into(),
                ano: "2024".into(),
            })
            .collect(),
    };
    let resultado = reunir([pagina(3, 1, 2), pagina(3, 3, 3)]).unwrap();
    assert_eq!(resultado.len(), 3);
    assert_eq!(resultado[0], resultado[2]);
    for paginas in [
        vec![pagina(3, 1, 2)],
        vec![pagina(3, 1, 2), pagina(3, 1, 2)],
        vec![pagina(3, 1, 1), pagina(3, 3, 3)],
        vec![pagina(3, 1, 2), pagina(4, 3, 4)],
        vec![pagina(3, 1, 3), pagina(3, 1, 3)],
        vec![pagina(3, 2, 3)],
    ] {
        assert_eq!(reunir(paginas), Err(ErroAcervo::FaixaIncompleta));
    }
    let zero = || Pagina {
        legenda: Legenda {
            total: 0,
            faixa: None,
        },
        processos: Vec::new(),
    };
    assert_eq!(reunir([zero()]).unwrap(), [] as [ProcessoDoAcervo; 0]);
    assert_eq!(reunir([zero(), zero()]), Err(ErroAcervo::FaixaIncompleta));
    assert_eq!(reunir([]), Err(ErroAcervo::FaixaIncompleta));
}

fn conferir_campos(post: &crate::formulario_web::SubmissaoDerivada) {
    for nome in ["selOrgao", "txtPeriodoDe", "txtPeriodoA", "sbmPesquisar"] {
        assert_eq!(
            post.nomes_dos_campos()
                .iter()
                .filter(|n| n.as_str() == nome)
                .count(),
            1
        );
    }
}

/// Somente diagnóstico do leitor de detalhe, não execução da CLI nem prova
/// de que todas as respostas pertencem à mesma fotografia do acervo.
#[test]
#[ignore = "acervo original: exige ZIPs verificados no cache"]
fn acervo_detalhe_original() {
    use crate::cassette::acervo::gravacao;
    use crate::cassette::{ZipDoAcervo, conferir_cassette_ate, html_da_resposta};
    use std::path::{Path, PathBuf};

    let raiz = Path::new(env!("CARGO_MANIFEST_DIR")).join("cassettes");
    let cache = std::env::var_os("IES_ACERVO_CACHE").map_or_else(
        || PathBuf::from(std::env::var_os("HOME").expect("HOME")).join(".cache/ies"),
        PathBuf::from,
    );
    for (instalacao, revisao, total, paginas, retornos) in [
        ("unb", "74b641c4a8a1", 21, 1, 0),
        ("pen1", "5c557e93a6ce", 305, 7, 1),
        ("pen2", "c64c576f9a9a", 685, 14, 1),
    ] {
        let card = raiz.join(instalacao).join(format!(
            "unidade-estatisticas-acervo-{instalacao}--{revisao}.md"
        ));
        let arquivo = crate::cassette::archive_filename(&card).expect("card OKF");
        let zip = ZipDoAcervo::ler(&cache.join(&arquivo)).expect("ZIP original");
        let conferida =
            conferir_cassette_ate(&card, &zip, 120_000_000).expect("integridade do original");
        assert_eq!(conferida.instalacao, instalacao);
        let mut lidas = Vec::new();
        let mut formulario_derivado = false;
        let instalacao_gravada = gravacao::instalacao(&conferida.json).expect("instalacao gravada");
        let mut total_anterior = None;
        let mut unidade = None;
        for resposta in gravacao::respostas(&conferida.json) {
            if resposta.operacao == "procedimento_controlar" && resposta.status == 200 {
                let html = html_da_resposta(resposta.interacao).expect("Controle original");
                unidade = Some(
                    crate::controle_contexto::unidade_exibida_no_controle(&html)
                        .expect("unidade do Controle original"),
                );
            }
            if resposta.operacao == "gerar_estatisticas_unidade"
                && resposta.status == 200
                && lidas.is_empty()
            {
                let html = html_da_resposta(resposta.interacao).expect("HTML original");
                let tela = HtmlServido::de_teste(&html);
                if resposta.metodo == "GET" && !formulario_derivado {
                    let dom = arvore(&tela);
                    if crate::formulario_web::localizar_formulario(&dom, "frmEstatisticas").is_ok()
                    {
                        let url = gravacao::url_sem_consulta(&conferida.json, resposta.interacao);
                        let codificacao = crate::pagina_do_controle::codificacao_da_tela(
                            &gravacao::mime(resposta.interacao),
                        )
                        .expect("charset original");
                        let servida = crate::formulario_web::TelaServida {
                            url: &url,
                            html: &tela,
                            codificacao,
                        };
                        let data = crate::cli_unidade_aux::data_do_acervo("2024-02-29").unwrap();
                        let post = formulario_do_acervo(
                            &instalacao_gravada,
                            &servida,
                            unidade.as_deref().expect("unidade do Controle anterior"),
                            data,
                        )
                        .expect("derivacao do formulario original");
                        conferir_campos(&post);
                        formulario_derivado = true;
                    }
                }
                if let Ok((n, _)) = total_do_resultado(&tela, &instalacao_gravada) {
                    total_anterior = Some(n);
                }
            }
            if resposta.operacao != "estatisticas_detalhar_unidade" || resposta.status != 200 {
                continue;
            }
            let html = html_da_resposta(resposta.interacao).expect("HTML original");
            assert_eq!(
                resposta.metodo,
                if lidas.is_empty() { "GET" } else { "POST" },
                "{instalacao}: metodo do detalhe"
            );
            let tela = HtmlServido::de_teste(&html);
            if lidas.is_empty() {
                paginacao_tests::observar_formularios(
                    instalacao,
                    &tela,
                    &conferida.json,
                    total_anterior.expect("total do relatório anterior"),
                    &instalacao_gravada,
                    unidade.as_deref().expect("unidade do Controle anterior"),
                );
            }
            lidas.push(detalhe(&tela).expect("detalhe original"));
        }
        assert!(formulario_derivado, "{instalacao}: formulario GET ausente");
        assert_eq!(
            total_anterior,
            Some(total),
            "{instalacao}: total do relatório"
        );
        conferir_travessia_original(instalacao, lidas, total, paginas, retornos);
    }
}

fn conferir_travessia_original(
    instalacao: &str,
    mut lidas: Vec<Pagina>,
    total: usize,
    paginas: usize,
    retornos: usize,
) {
    // Os originais PEN1/PEN2 voltam à primeira página depois do fim.
    // Essa navegação não pertence à travessia de coleta.
    assert_eq!(lidas.len(), paginas + retornos, "{instalacao}");
    let voltas = lidas.split_off(paginas);
    for volta in voltas {
        assert_eq!(volta.legenda, lidas[0].legenda, "{instalacao}");
    }
    let processos = reunir(lidas).expect("cobertura integral da primeira travessia");
    assert_eq!(processos.len(), total, "{instalacao}");
    let distintos = processos
        .iter()
        .map(|p| p.processo.as_str())
        .collect::<std::collections::HashSet<_>>()
        .len();
    println!("{instalacao}: cobertura={total} NUPs_distintos={distintos}");
}
