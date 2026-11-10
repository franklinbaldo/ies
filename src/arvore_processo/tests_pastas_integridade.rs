//! RED/GREEN da integridade das pastas (por identidade, não por contagem)
//! e da ação "Abrir todas as Pastas". Árvores sintéticas de
//! [`super::tests_pastas`].
use super::tests::NUP;
use super::tests_pastas::*;
use super::*;
use crate::erro_html::ErroHtml;

fn p(k: usize) -> String {
    format!("PASTA{k}")
}

/// Declarações `Pastas[k]['protocolos']` de um caso.
type Declaradas<'a> = Vec<(usize, &'a [&'a str])>;

fn ler(html: &str) -> Result<ArvoreLida, ErroArvore> {
    ler_arvore(html, NUP, ID)
}

/// Árvore aberta com as declarações e os documentos dados.
fn aberta(declaradas: &[(usize, &[&str])], documentos: Vec<No>) -> String {
    let mut nos = vec![No::Processo, No::Pasta(1, true), No::Pasta(2, true)];
    nos.extend(documentos);
    arvore_em_pastas(
        &declaracoes(declaradas),
        &nos,
        &acoes_das_pastas(&url_abrir(ID)),
    )
}

fn os_quatro() -> Vec<No> {
    vec![
        No::Documento("21", p(1), Some("900")),
        No::Documento("22", p(1), Some("901")),
        No::Documento("31", p(2), Some("902")),
        No::Documento("32", p(2), Some("903")),
    ]
}

#[test]
fn identidade_e_nao_contagem() {
    let declaradas: &[(usize, &[&str])] = &[(1, &PASTA1), (2, &PASTA2)];
    // Mesma contagem, protocolo trocado: não é a pasta declarada.
    let mut trocado = os_quatro();
    trocado[1] = No::Documento("22", p(1), Some("999"));
    assert_eq!(
        ler(&aberta(declaradas, trocado)),
        Err(super::ErroArvore::DocumentoDaPastaNaoDeclarado)
    );
    // Um a menos.
    let mut faltando = os_quatro();
    faltando.remove(3);
    assert_eq!(
        ler(&aberta(declaradas, faltando)),
        Err(super::ErroArvore::Html(ErroHtml::ausente(
            "protocolo_declarado_ausente_da_pasta"
        )))
    );
    // O mesmo protocolo duas vezes, com a contagem certa.
    let mut repetido = os_quatro();
    repetido[1] = No::Documento("22", p(1), Some("900"));
    assert_eq!(
        ler(&aberta(declaradas, repetido)),
        Err(super::ErroArvore::Html(ErroHtml::repetido(
            "protocolo_repetido_na_pasta"
        )))
    );
    // Protocolo de outra pasta.
    let mut de_outra = os_quatro();
    de_outra[0] = No::Documento("21", p(1), Some("902"));
    assert_eq!(
        ler(&aberta(declaradas, de_outra)),
        Err(super::ErroArvore::DocumentoDaPastaNaoDeclarado)
    );
    // Nenhuma linha `.acoes` para o nó: nunca observado, continua recusa.
    let sem_linha = aberta(declaradas, os_quatro()).replace(
        &format!("{}\n", super::tests_pastas::barra(5, "31", Some("902"))),
        "",
    );
    assert_eq!(
        ler(&sem_linha),
        Err(super::ErroArvore::DocumentoDaPastaSemIdentidadeNaBarra)
    );
}

#[test]
fn declaracoes_das_pastas_conferidas() {
    let casos: Vec<(Declaradas, super::ErroArvore)> = vec![
        (
            vec![(1, &PASTA1)],
            super::ErroArvore::PastaSemProtocolosDeclarados,
        ),
        (
            vec![(1, &PASTA1), (2, &PASTA2), (3, &["904"])],
            super::ErroArvore::Html(ErroHtml::ausente(
                "protocolos_declarados_para_pasta_ausente",
            )),
        ),
        (
            vec![(1, &PASTA1), (2, &PASTA2), (2, &PASTA2)],
            super::ErroArvore::Html(ErroHtml::repetido(
                "protocolos_da_pasta_declarados_mais_de_uma_vez",
            )),
        ),
        (
            vec![(1, &["900", "x"]), (2, &PASTA2)],
            super::ErroArvore::Html(ErroHtml::valor(
                "protocolos_da_pasta_fora_da_forma_observada",
            )),
        ),
        (
            vec![(1, &["900", "900"]), (2, &PASTA2)],
            super::ErroArvore::Html(ErroHtml::valor(
                "protocolos_da_pasta_fora_da_forma_observada",
            )),
        ),
        (
            vec![(1, &[]), (2, &PASTA2)],
            super::ErroArvore::Html(ErroHtml::valor(
                "protocolos_da_pasta_fora_da_forma_observada",
            )),
        ),
    ];
    for (declaradas, esperado) in casos {
        assert_eq!(
            ler(&aberta(&declaradas, os_quatro())),
            Err(esperado),
            "{declaradas:?}"
        );
    }
    let mal = aberta(&[(1, &PASTA1), (2, &PASTA2)], os_quatro()).replace(
        "Pastas[2]['protocolos'] = '",
        "Pastas[2]['protocolos'] = x + '",
    );
    assert_eq!(
        ler(&mal),
        Err(super::ErroArvore::Html(ErroHtml::valor(
            "protocolos_da_pasta_fora_da_forma_observada"
        )))
    );
}

#[test]
fn nos_de_pasta_na_forma_observada() {
    let base = arvore_aberta();
    let casos = [
        // 16 argumentos numa pasta.
        (
            base.replace(
                ",true,true,null,null,null);\nNos[1]",
                ",true,true,null,null,null,null);\nNos[1]",
            ),
            super::ErroArvore::QuantidadeDeArgumentosDoNoDiferenteDaObservada,
        ),
        // Pai que não é o processo.
        (
            base.replace("\"PASTA1\",\"10\"", "\"PASTA1\",\"11\""),
            super::ErroArvore::PastaComPaiDiferenteDoProcesso,
        ),
        // Ação diferente de abrirFecharPasta da própria pasta.
        (
            base.replace("abrirFecharPasta('PASTA1')", "abrirFecharPasta('PASTA2')"),
            super::ErroArvore::Html(ErroHtml::valor("pasta_fora_da_forma_observada")),
        ),
        (
            base.replace("\"PASTA\",\"PASTA1\"", "\"PASTA\",\"P1\""),
            super::ErroArvore::Html(ErroHtml::valor("pasta_fora_da_forma_observada")),
        ),
        // A mesma pasta duas vezes.
        (
            base.replace(
                "\"PASTA2\",\"10\",\"javascript:abrirFecharPasta('PASTA2');\"",
                "\"PASTA1\",\"10\",\"javascript:abrirFecharPasta('PASTA1');\"",
            ),
            super::ErroArvore::Html(ErroHtml::repetido("pasta_repetida_na_arvore")),
        ),
        // Documento numa pasta que não existe.
        (
            base.replace("\"32\",\"PASTA2\"", "\"32\",\"PASTA7\""),
            super::ErroArvore::DocumentoComPaiDiferenteDoProcesso,
        ),
    ];
    for (html, esperado) in casos {
        assert_eq!(ler(&html), Err(esperado));
    }
}

#[test]
fn aguarde_na_forma_observada() {
    let base = arvore_fechada();
    assert_eq!(
        ler(&base.replace("\"AGUARDE1\",\"PASTA1\"", "\"AGUARDE1\",\"PASTA2\"")),
        Err(super::ErroArvore::Html(ErroHtml::valor(
            "aguarde_fora_da_forma_observada"
        )))
    );
    assert_eq!(
        ler(&base.replace("\"AGUARDE1\",\"PASTA1\"", "\"AGUARDE7\",\"PASTA7\"")),
        Err(super::ErroArvore::AguardeComPaiDiferenteDePasta)
    );
    // Pasta fechada que também traz documento: nunca observada.
    let metade = arvore_em_pastas(
        &declaracoes(&[(1, &PASTA1), (2, &PASTA2)]),
        &[
            No::Processo,
            No::Pasta(1, false),
            No::Aguarde(1),
            No::Documento("21", p(1), Some("900")),
            No::Pasta(2, true),
            No::Documento("31", p(2), Some("902")),
            No::Documento("32", p(2), Some("903")),
        ],
        "",
    );
    assert_eq!(
        ler(&metade),
        Err(super::ErroArvore::PastaCarregadaPelaMetade)
    );
    // A pasta aberta da árvore fechada também é conferida.
    let errada = base.replace(
        "id_rel_protocolo_protocolo=903",
        "id_rel_protocolo_protocolo=999",
    );
    assert_eq!(
        ler(&errada),
        Err(super::ErroArvore::DocumentoDaPastaNaoDeclarado)
    );
}

#[test]
fn barra_de_documento_de_pasta_fora_da_forma_e_recusa() {
    let html = arvore_aberta().replace("Nos[3].acoes = '", "Nos[3].acoes = x + '");
    assert_eq!(
        ler(&html),
        Err(super::ErroArvore::Html(ErroHtml::valor(
            "barra_de_no_fora_da_forma_observada"
        )))
    );
}

#[test]
fn identidade_do_processo_vem_antes_das_pastas() {
    // Árvore de outro processo: o palpite é esquecido, não vira recusa de pasta.
    let outra = arvore_fechada().replace(
        "id_rel_protocolo_protocolo=903",
        "id_rel_protocolo_protocolo=999",
    );
    assert_eq!(
        ler_arvore(&outra, "00002.000002/2026-02", ID),
        Err(super::ErroArvore::IdentidadeDoProcessoDiferenteDaPedida)
    );
}

fn instalacao() -> crate::instalacao::Instalacao {
    crate::instalacao::Instalacao::interpretar("https://sei.exemplo.gov.br/sei").unwrap()
}

#[test]
fn acao_de_abrir_pastas_lida_e_conferida() {
    let inst = instalacao();
    let endereco = ler_acao_de_abrir_pastas(&inst, &arvore_fechada(), ID)
        .unwrap()
        .unwrap();
    assert_eq!(
        endereco.as_str(),
        format!("https://sei.exemplo.gov.br/sei/{}", url_abrir(ID))
    );
    assert!(!format!("{endereco:?}").contains(&"f".repeat(64)));
    // Sem a ação: nada (quem chama decide a recusa).
    let sem = arvore_fechada().replace("\"ABRIR_PASTAS\"", "\"OUTRA_ACAO\"");
    assert_eq!(ler_acao_de_abrir_pastas(&inst, &sem, ID), Ok(None));
}

#[test]
fn acao_de_abrir_pastas_fora_da_forma_e_recusa() {
    let inst = instalacao();
    let url = url_abrir(ID);
    let com = |acoes: String| {
        arvore_em_pastas(
            &declaracoes(&[(1, &PASTA1), (2, &PASTA2)]),
            &[
                No::Processo,
                No::Pasta(1, false),
                No::Aguarde(1),
                No::Pasta(2, false),
                No::Aguarde(2),
            ],
            &acoes,
        )
    };
    let casos = [
        (
            url.replace("abrir_pastas=1", "abrir_pastas=0"),
            super::ErroArvore::Html(ErroHtml::valor("endereco_de_abrir_pastas_nao_reconhecido")),
        ),
        (
            url.replace("&abrir_pastas=1", ""),
            super::ErroArvore::Html(ErroHtml::valor("endereco_de_abrir_pastas_nao_reconhecido")),
        ),
        (
            format!("{url}&fechar_pastas=1"),
            super::ErroArvore::Html(ErroHtml::valor("endereco_de_abrir_pastas_nao_reconhecido")),
        ),
        (
            format!("{url}&abrir_pastas=1"),
            super::ErroArvore::Html(ErroHtml::valor("endereco_de_abrir_pastas_nao_reconhecido")),
        ),
        (
            url.replace("id_procedimento=10", "id_procedimento=11"),
            super::ErroArvore::Html(ErroHtml::valor("endereco_de_abrir_pastas_nao_reconhecido")),
        ),
        (
            url.replace(
                "acao=procedimento_visualizar&",
                "acao=procedimento_paginar&",
            ),
            super::ErroArvore::Html(ErroHtml::valor("endereco_de_abrir_pastas_nao_reconhecido")),
        ),
        (
            url.replace(&"f".repeat(64), "f"),
            super::ErroArvore::Html(ErroHtml::valor("endereco_de_abrir_pastas_nao_reconhecido")),
        ),
        (
            format!("https://outro.exemplo/{url}"),
            super::ErroArvore::Html(ErroHtml::valor("endereco_de_abrir_pastas_nao_reconhecido")),
        ),
    ];
    for (endereco, esperado) in casos {
        assert_eq!(
            ler_acao_de_abrir_pastas(&inst, &com(acoes_das_pastas(&endereco)), ID).map(|_| ()),
            Err(esperado),
            "{endereco}"
        );
    }
    let duas = format!("{}{}", acoes_das_pastas(&url), acoes_das_pastas(&url));
    assert_eq!(
        ler_acao_de_abrir_pastas(&inst, &com(duas), ID).map(|_| ()),
        Err(super::ErroArvore::Html(ErroHtml::repetido(
            "acoes_de_abrir_pastas_repetidas"
        )))
    );
    for mal in [
        acoes_das_pastas(&url).replace("\"AP10\"", "\"AP11\""),
        acoes_das_pastas(&url).replace("\"AP10\",\"10\"", "\"AP10\",\"11\""),
        acoes_das_pastas(&url).replace(",\"/infra_css/svg/mais.svg\",true);", ",true);"),
        acoes_das_pastas(&url).replace("NosAcoes[1] = new", "NosAcoes[1] = x || new"),
    ] {
        assert_eq!(
            ler_acao_de_abrir_pastas(&inst, &com(mal), ID).map(|_| ()),
            Err(super::ErroArvore::Html(ErroHtml::valor(
                "acao_de_abrir_pastas_fora_da_forma_observada"
            )))
        );
    }
}

fn vazios(k: usize, ids: &[&'static str]) -> Vec<No> {
    ids.iter().map(|id| No::Documento(id, p(k), None)).collect()
}

#[test]
fn barras_todas_vazias_sao_conferidas_pela_contagem() {
    use super::VerificacaoDaPasta::{PorContagem, PorIdentidade};
    let declaradas: &[(usize, &[&str])] = &[(1, &PASTA1), (2, &PASTA2)];
    let mut todos = vazios(1, &["21", "22"]);
    todos.extend(vazios(2, &["31", "32"]));
    let lida = ler(&aberta(declaradas, todos)).unwrap();
    assert_eq!(
        lida.pastas
            .iter()
            .map(|p| p.verificacao)
            .collect::<Vec<_>>(),
        [Some(PorContagem), Some(PorContagem)]
    );
    assert_eq!(
        conferencia(&lida),
        [(Some(PorContagem), 0, 2), (Some(PorContagem), 0, 2)]
    );
    assert_eq!(lida.documentos.len(), 4);
    // Cada pasta escolhe a sua regra: a 1 por identidade, a 2 por contagem.
    let mut mistas = os_quatro();
    mistas.truncate(2);
    mistas.extend(vazios(2, &["31", "32"]));
    let lida = ler(&aberta(declaradas, mistas)).unwrap();
    assert_eq!(
        lida.pastas
            .iter()
            .map(|p| p.verificacao)
            .collect::<Vec<_>>(),
        [Some(PorIdentidade), Some(PorContagem)]
    );
}

#[test]
fn contagem_diferente_da_declarada_e_recusa() {
    let declaradas: &[(usize, &[&str])] = &[(1, &PASTA1), (2, &PASTA2)];
    for segunda in [vec!["31"], vec!["31", "32", "33"]] {
        let mut nos = os_quatro();
        nos.truncate(2);
        nos.extend(vazios(2, &segunda));
        assert_eq!(
            ler(&aberta(declaradas, nos)),
            Err(super::ErroArvore::ContagemDaPastaDiferenteDaDeclarada),
            "{segunda:?}"
        );
    }
    // Id de documento repetido na árvore continua recusa.
    let mut nos = vazios(1, &["21", "22"]);
    nos.extend(vazios(2, &["31", "21"]));
    assert_eq!(
        ler(&aberta(declaradas, nos)),
        Err(super::ErroArvore::Html(ErroHtml::repetido(
            "documento_repetido_na_arvore"
        )))
    );
}

fn conferencia(lida: &ArvoreLida) -> Vec<(Option<VerificacaoDaPasta>, usize, usize)> {
    lida.pastas
        .iter()
        .map(|p| {
            (
                p.verificacao,
                p.conferidos_por_identidade,
                p.conferidos_por_contagem,
            )
        })
        .collect()
}

#[test]
fn identidade_por_documento_e_o_resto_pela_contagem() {
    use super::VerificacaoDaPasta::{Mista, PorIdentidade};
    let declaradas: &[(usize, &[&str])] = &[(1, &PASTA1), (2, &PASTA2)];
    // Barra vazia ou com links sem `id_rel_protocolo_protocolo`: sem identidade.
    for sem in [None, Some("-")] {
        let mut nos = os_quatro();
        nos[2] = No::Documento("31", p(2), sem);
        let lida = ler(&aberta(declaradas, nos)).unwrap();
        assert_eq!(
            conferencia(&lida),
            [(Some(PorIdentidade), 2, 0), (Some(Mista), 1, 1)],
            "{sem:?}"
        );
    }
}

#[test]
fn pasta_mista_que_nao_fecha_e_recusa() {
    let declaradas: &[(usize, &[&str])] = &[(1, &PASTA1), (2, &PASTA2)];
    let com_pasta2 = |segunda: Vec<No>| {
        let mut nos = os_quatro();
        nos.truncate(2);
        nos.extend(segunda);
        ler(&aberta(declaradas, nos))
    };
    // 1 identidade + 2 sem: 2 declarados − 1 = 1 ≠ 2.
    assert_eq!(
        com_pasta2(vec![
            No::Documento("31", p(2), Some("902")),
            No::Documento("32", p(2), None),
            No::Documento("33", p(2), Some("-")),
        ]),
        Err(super::ErroArvore::ContagemDaPastaDiferenteDaDeclarada)
    );
    // A identidade que aparece continua conferida.
    assert_eq!(
        com_pasta2(vec![
            No::Documento("31", p(2), Some("999")),
            No::Documento("32", p(2), None),
        ]),
        Err(super::ErroArvore::DocumentoDaPastaNaoDeclarado)
    );
    assert_eq!(
        com_pasta2(vec![
            No::Documento("31", p(2), Some("902")),
            No::Documento("32", p(2), Some("902")),
            No::Documento("33", p(2), None),
        ]),
        Err(super::ErroArvore::Html(ErroHtml::repetido(
            "protocolo_repetido_na_pasta"
        )))
    );
    // Toda por identidade e faltando um: o erro de identidade.
    assert_eq!(
        com_pasta2(vec![No::Documento("31", p(2), Some("902"))]),
        Err(super::ErroArvore::Html(ErroHtml::ausente(
            "protocolo_declarado_ausente_da_pasta"
        )))
    );
}
