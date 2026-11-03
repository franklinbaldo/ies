//! RED/GREEN da integridade das pastas (por identidade, não por contagem)
//! e da ação "Abrir todas as Pastas". Árvores sintéticas de
//! [`super::tests_pastas`].
use super::ErroArvore as E;
use super::tests::NUP;
use super::tests_pastas::*;
use super::*;

fn p(k: usize) -> String {
    format!("PASTA{k}")
}

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
        Err(E::DocumentoDaPastaNaoDeclarado)
    );
    // Um a menos.
    let mut faltando = os_quatro();
    faltando.remove(3);
    assert_eq!(
        ler(&aberta(declaradas, faltando)),
        Err(E::ProtocoloDeclaradoAusenteDaPasta)
    );
    // O mesmo protocolo duas vezes, com a contagem certa.
    let mut repetido = os_quatro();
    repetido[1] = No::Documento("22", p(1), Some("900"));
    assert_eq!(
        ler(&aberta(declaradas, repetido)),
        Err(E::ProtocoloRepetidoNaPasta)
    );
    // Protocolo de outra pasta.
    let mut de_outra = os_quatro();
    de_outra[0] = No::Documento("21", p(1), Some("902"));
    assert_eq!(
        ler(&aberta(declaradas, de_outra)),
        Err(E::DocumentoDaPastaNaoDeclarado)
    );
    // Barra vazia (sem identidade): não se aceita pela contagem.
    let mut sem_barra = os_quatro();
    sem_barra[2] = No::Documento("31", p(2), None);
    assert_eq!(
        ler(&aberta(declaradas, sem_barra)),
        Err(E::DocumentoDaPastaSemIdentidadeNaBarra)
    );
}

#[test]
fn declaracoes_das_pastas_conferidas() {
    let casos: Vec<(Vec<(usize, &[&str])>, E)> = vec![
        (vec![(1, &PASTA1)], E::PastaSemProtocolosDeclarados),
        (
            vec![(1, &PASTA1), (2, &PASTA2), (3, &["904"])],
            E::ProtocolosDeclaradosParaPastaAusente,
        ),
        (
            vec![(1, &PASTA1), (2, &PASTA2), (2, &PASTA2)],
            E::ProtocolosDaPastaDeclaradosMaisDeUmaVez,
        ),
        (
            vec![(1, &["900", "x"]), (2, &PASTA2)],
            E::ProtocolosDaPastaForaDaFormaObservada,
        ),
        (
            vec![(1, &["900", "900"]), (2, &PASTA2)],
            E::ProtocolosDaPastaForaDaFormaObservada,
        ),
        (
            vec![(1, &[]), (2, &PASTA2)],
            E::ProtocolosDaPastaForaDaFormaObservada,
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
    assert_eq!(ler(&mal), Err(E::ProtocolosDaPastaForaDaFormaObservada));
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
            E::QuantidadeDeArgumentosDoNoDiferenteDaObservada,
        ),
        // Pai que não é o processo.
        (
            base.replace("\"PASTA1\",\"10\"", "\"PASTA1\",\"11\""),
            E::PastaComPaiDiferenteDoProcesso,
        ),
        // Ação diferente de abrirFecharPasta da própria pasta.
        (
            base.replace("abrirFecharPasta('PASTA1')", "abrirFecharPasta('PASTA2')"),
            E::PastaForaDaFormaObservada,
        ),
        (
            base.replace("\"PASTA\",\"PASTA1\"", "\"PASTA\",\"P1\""),
            E::PastaForaDaFormaObservada,
        ),
        // A mesma pasta duas vezes.
        (
            base.replace(
                "\"PASTA2\",\"10\",\"javascript:abrirFecharPasta('PASTA2');\"",
                "\"PASTA1\",\"10\",\"javascript:abrirFecharPasta('PASTA1');\"",
            ),
            E::PastaRepetidaNaArvore,
        ),
        // Documento numa pasta que não existe.
        (
            base.replace("\"32\",\"PASTA2\"", "\"32\",\"PASTA7\""),
            E::DocumentoComPaiDiferenteDoProcesso,
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
        Err(E::AguardeForaDaFormaObservada)
    );
    assert_eq!(
        ler(&base.replace("\"AGUARDE1\",\"PASTA1\"", "\"AGUARDE7\",\"PASTA7\"")),
        Err(E::AguardeComPaiDiferenteDePasta)
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
    assert_eq!(ler(&metade), Err(E::PastaCarregadaPelaMetade));
    // A pasta aberta da árvore fechada também é conferida.
    let errada = base.replace(
        "id_rel_protocolo_protocolo=903",
        "id_rel_protocolo_protocolo=999",
    );
    assert_eq!(ler(&errada), Err(E::DocumentoDaPastaNaoDeclarado));
}

#[test]
fn barra_de_documento_de_pasta_fora_da_forma_e_recusa() {
    let html = arvore_aberta().replace("Nos[3].acoes = '", "Nos[3].acoes = x + '");
    assert_eq!(ler(&html), Err(E::BarraDeNoForaDaFormaObservada));
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
        Err(E::IdentidadeDoProcessoDiferenteDaPedida)
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
            E::EnderecoDeAbrirPastasNaoReconhecido,
        ),
        (
            url.replace("&abrir_pastas=1", ""),
            E::EnderecoDeAbrirPastasNaoReconhecido,
        ),
        (
            format!("{url}&fechar_pastas=1"),
            E::EnderecoDeAbrirPastasNaoReconhecido,
        ),
        (
            format!("{url}&abrir_pastas=1"),
            E::EnderecoDeAbrirPastasNaoReconhecido,
        ),
        (
            url.replace("id_procedimento=10", "id_procedimento=11"),
            E::EnderecoDeAbrirPastasNaoReconhecido,
        ),
        (
            url.replace(
                "acao=procedimento_visualizar&",
                "acao=procedimento_paginar&",
            ),
            E::EnderecoDeAbrirPastasNaoReconhecido,
        ),
        (
            url.replace(&"f".repeat(64), "f"),
            E::EnderecoDeAbrirPastasNaoReconhecido,
        ),
        (
            format!("https://outro.exemplo/{url}"),
            E::EnderecoDeAbrirPastasNaoReconhecido,
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
        Err(E::AcoesDeAbrirPastasRepetidas)
    );
    for mal in [
        acoes_das_pastas(&url).replace("\"AP10\"", "\"AP11\""),
        acoes_das_pastas(&url).replace("\"AP10\",\"10\"", "\"AP10\",\"11\""),
        acoes_das_pastas(&url).replace(",\"/infra_css/svg/mais.svg\",true);", ",true);"),
        acoes_das_pastas(&url).replace("NosAcoes[1] = new", "NosAcoes[1] = x || new"),
    ] {
        assert_eq!(
            ler_acao_de_abrir_pastas(&inst, &com(mal), ID).map(|_| ()),
            Err(E::AcaoDeAbrirPastasForaDaFormaObservada)
        );
    }
}
