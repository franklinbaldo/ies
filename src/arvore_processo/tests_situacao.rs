//! RED/GREEN de `aberto_em`: o `.html` do nó `PROCESSO` nas formas
//! observadas na `UnB` 4.0.9, em GO 5.0.3 e na PEN1 5.0.4 (conteúdo das
//! telas de treinamento; nenhuma URL ou hash), e as formas recusadas, que
//! viram "não lido" sem falhar a árvore.
use super::situacao::{ler_aberto_em, ler_sobrestamento, ler_unidades};
use super::tests::{NUP, arvore, no_documento, no_processo};
use super::tests_anexado_a::{ID_PAI, NUP_PAI, href_do_pai, html_do_anexado};
use super::*;
use crate::tela_html::HtmlServido;

fn ancora(nome: &str, sigla: &str) -> String {
    format!("<a alt=\"{nome}\" title=\"{nome}\" class=\"ancoraSigla\">{sigla}</a>")
}

fn unidade(sigla: &str, nome: &str, atribuido_a: Option<(&str, &str)>) -> UnidadeAberta {
    UnidadeAberta {
        sigla: sigla.into(),
        nome: nome.into(),
        atribuido_a: atribuido_a.map(|(sigla, nome)| Atribuicao {
            sigla: sigla.into(),
            nome: nome.into(),
        }),
    }
}

fn somente(ancora_da_unidade: &str) -> String {
    format!("Processo aberto somente na unidade {ancora_da_unidade}.<br />")
}

/// Forma plural da `UnB` (quatro unidades, nenhuma atribuída).
fn plural_unb() -> String {
    format!(
        "Processo aberto nas unidades:<br />{}<br />{}<br />{}<br />{}<br /><br />",
        ancora("Arquivo Central", "ACE"),
        ancora("Unidade2", "Unidade2"),
        ancora("Unidade3", "Unidade3"),
        ancora("Unidade4", "Unidade4"),
    )
}

/// Forma plural de GO: siglas com `/`, `-` e dígitos, nomes com acento e
/// uma unidade atribuída.
fn plural_go() -> String {
    format!(
        "Processo aberto nas unidades:<br />{}<br />{} (atribuído para {})<br />{}<br /><br />",
        ancora("GIRÊNCUO DI TICNALAGUO", "CGE/GETEC-18153"),
        ancora("CANSILHA DI DISINVALVUMINTA DA ISTODA", "SEAD/CDE-09516"),
        ancora("Xlxnx trxxnxmxntx 05", "aluno05"),
        ancora(
            "GIRÊNCUO DI GISTÃA I DISINVALVUMINTA DI PISSAOS",
            "SEAD/GGP-05610"
        ),
    )
}

#[test]
fn somente_numa_unidade_sem_e_com_atribuicao() {
    assert_eq!(
        ler_unidades(&HtmlServido::de_teste(&somente(&ancora(
            "Unidade1", "Unidade1"
        )))),
        Some(vec![unidade("Unidade1", "Unidade1", None)])
    );
    assert_eq!(
        ler_unidades(&HtmlServido::de_teste(&somente(&ancora(
            "Unidade de Teste 1",
            "TESTE"
        )))),
        Some(vec![unidade("TESTE", "Unidade de Teste 1", None)])
    );
    let atribuida = HtmlServido::de_teste(&format!(
        "Processo aberto somente na unidade {} (atribuído para {}).<br />",
        ancora("Unidade1", "Unidade1"),
        ancora("aluno2", "aluno2")
    ));
    assert_eq!(
        ler_unidades(&atribuida),
        Some(vec![unidade(
            "Unidade1",
            "Unidade1",
            Some(("aluno2", "aluno2"))
        )])
    );
}

#[test]
fn nas_unidades_na_ordem_servida_com_atribuicao_por_unidade() {
    assert_eq!(
        ler_unidades(&HtmlServido::de_teste(&plural_unb())),
        Some(vec![
            unidade("ACE", "Arquivo Central", None),
            unidade("Unidade2", "Unidade2", None),
            unidade("Unidade3", "Unidade3", None),
            unidade("Unidade4", "Unidade4", None),
        ])
    );
    assert_eq!(
        ler_unidades(&HtmlServido::de_teste(&plural_go())),
        Some(vec![
            unidade("CGE/GETEC-18153", "GIRÊNCUO DI TICNALAGUO", None),
            unidade(
                "SEAD/CDE-09516",
                "CANSILHA DI DISINVALVUMINTA DA ISTODA",
                Some(("aluno05", "Xlxnx trxxnxmxntx 05"))
            ),
            unidade(
                "SEAD/GGP-05610",
                "GIRÊNCUO DI GISTÃA I DISINVALVUMINTA DI PISSAOS",
                None
            ),
        ])
    );
}

#[test]
fn sem_andamentos_abertos_e_lista_vazia() {
    assert_eq!(
        ler_unidades(&HtmlServido::de_teste(
            "Processo não possui andamentos abertos.<br />"
        )),
        Some(Vec::new())
    );
}

#[test]
fn entidade_no_nome_e_decodificada() {
    assert_eq!(
        ler_unidades(&HtmlServido::de_teste(&somente(&ancora(
            "Arquivo &amp; Protocolo",
            "AP"
        )))),
        Some(vec![unidade("AP", "Arquivo & Protocolo", None)])
    );
}

#[test]
fn formas_nao_observadas_nao_sao_lidas() {
    let u1 = ancora("Unidade1", "Unidade1");
    let casos = [
        String::new(),
        html_do_anexado(&href_do_pai(ID_PAI, "10"), NUP_PAI),
        // Sem o ponto, sem o `<br />`, com sobra ou com outra frase.
        format!("Processo aberto somente na unidade {u1}<br />"),
        format!("Processo aberto somente na unidade {u1}."),
        format!("{}x", somente(&u1)),
        format!("Processo aberto apenas na unidade {u1}.<br />"),
        "Processo não possui andamentos abertos.".to_owned(),
        "Processo não possui andamentos abertos.<br /><br />".to_owned(),
        // Âncora fora da forma: sem alt, alt ≠ title, outra classe, href,
        // entidade ou tag no texto, texto vazio ou com espaço.
        somente("<a class=\"ancoraSigla\">U</a>"),
        somente("<a alt=\"A\" title=\"B\" class=\"ancoraSigla\">U</a>"),
        somente("<a alt=\"A\" title=\"A\" class=\"outra\">U</a>"),
        somente("<a alt=\"A\" title=\"A\" class=\"ancoraSigla\" href=\"x\">U</a>"),
        somente("<a alt=\"A\" title=\"A\" class=\"ancoraSigla\">U&amp;V</a>"),
        somente("<a alt=\"A\" title=\"A\" class=\"ancoraSigla\"><b>U</b></a>"),
        somente("<a alt=\"A\" title=\"A\" class=\"ancoraSigla\"></a>"),
        somente("<a alt=\"A\" title=\"A\" class=\"ancoraSigla\"> U</a>"),
        somente("<a alt=\"\" title=\"\" class=\"ancoraSigla\">U</a>"),
        somente("<b alt=\"A\" title=\"A\" class=\"ancoraSigla\">U</b>"),
        // Plural sem unidade, sem o `<br />` final, com sigla repetida ou
        // atribuição sem o fecho.
        "Processo aberto nas unidades:<br /><br />".to_owned(),
        format!("Processo aberto nas unidades:<br />{u1}<br />"),
        format!("Processo aberto nas unidades:<br />{u1}<br />{u1}<br /><br />"),
        format!("Processo aberto nas unidades:<br />{u1} (atribuído para {u1}<br /><br />"),
        format!("Processo aberto nas unidades:<br /><!-- x -->{u1}<br /><br />"),
    ];
    for caso in casos {
        assert_eq!(ler_unidades(&HtmlServido::de_teste(&caso)), None, "{caso}");
    }
}

fn com_html(html: &str) -> String {
    arvore(&[
        no_processo(0, "10", NUP),
        no_documento(1, "21", "10", "0000101"),
    ])
    .replace("Nos[0].html = '';", &format!("Nos[0].html = '{html}';"))
}

#[test]
fn a_arvore_publica_aberto_em_do_html_da_raiz() {
    let lida = ler_arvore(&com_html(&plural_unb()), NUP, "10").unwrap();
    assert_eq!(
        lida.aberto_em,
        AbertoEm::Unidades(ler_unidades(&HtmlServido::de_teste(&plural_unb())).unwrap())
    );
    let mesma_linha = arvore(&[
        no_processo(0, "10", NUP),
        no_documento(1, "21", "10", "0000101"),
    ])
    .replace(
        "Nos[0].html = '';\n",
        "Nos[0].html = 'Processo não possui andamentos abertos.<br />';",
    );
    let lida = ler_arvore(&mesma_linha, NUP, "10").unwrap();
    assert_eq!(lida.aberto_em, AbertoEm::Unidades(Vec::new()));
    assert_eq!(lida.documentos.len(), 1);
}

#[test]
fn sem_html_ou_fora_da_forma_nao_e_lido_e_a_arvore_nao_falha() {
    use MotivoAbertoEm::{HtmlDaRaizForaDaFormaObservada, HtmlDaRaizNaoPublicado};
    let sem = com_html("").replace("Nos[0].html = '';\n", "");
    assert_eq!(
        ler_aberto_em(&sem),
        AbertoEm::NaoLido(HtmlDaRaizNaoPublicado)
    );
    let lida = ler_arvore(&com_html("Processo sobrestado.<br />"), NUP, "10").unwrap();
    assert_eq!(
        lida.aberto_em,
        AbertoEm::NaoLido(HtmlDaRaizForaDaFormaObservada)
    );
    assert_eq!(
        ler_aberto_em(&com_html("")),
        AbertoEm::NaoLido(HtmlDaRaizForaDaFormaObservada)
    );
}

#[test]
fn motivos_tem_nome_estavel() {
    assert_eq!(
        [
            MotivoAbertoEm::RaizPublicaOVinculoDeAnexacao,
            MotivoAbertoEm::RaizPublicaOSobrestamento,
            MotivoAbertoEm::HtmlDaRaizNaoPublicado,
            MotivoAbertoEm::HtmlDaRaizForaDaFormaObservada,
        ]
        .map(MotivoAbertoEm::como_texto),
        [
            "raiz_publica_o_vinculo_de_anexacao",
            "raiz_publica_o_sobrestamento",
            "html_da_raiz_nao_publicado",
            "html_da_raiz_fora_da_forma_observada",
        ]
    );
}

fn sobrestado(ancora_da_unidade: &str) -> String {
    format!("Processo sobrestado na unidade {ancora_da_unidade}.<br />")
}

fn sobrestada(sigla: &str, nome: &str) -> UnidadeSobrestada {
    UnidadeSobrestada {
        sigla: sigla.into(),
        nome: nome.into(),
    }
}

/// A frase das 21 árvores de `processo-sobrestar-retomar-*` (`UnB`, GO,
/// UFRJ e PEN1 a PEN4): uma unidade, sem atribuição.
#[test]
fn sobrestado_numa_unidade_nas_formas_do_acervo() {
    let casos = [
        ("Unidade1", "Unidade1"),
        ("TESTE", "Unidade de Teste 1"),
        ("CAXIAS", "Campus Duque de Caxias"),
        ("SEAD/CDE-09516", "CANSILHA DI DISINVALVUMINTA DA ISTODA"),
    ];
    for (sigla, nome) in casos {
        let html = sobrestado(&ancora(nome, sigla));
        assert_eq!(
            ler_sobrestamento(&HtmlServido::de_teste(&html)),
            Some(vec![sobrestada(sigla, nome)]),
            "{html}"
        );
        // Não é unidade aberta: a frase não diz onde o processo está aberto.
        assert_eq!(ler_unidades(&HtmlServido::de_teste(&html)), None, "{html}");
    }
    let lida = ler_arvore(
        &com_html(&sobrestado(&ancora("Unidade1", "Unidade1"))),
        NUP,
        "10",
    )
    .unwrap();
    assert_eq!(
        lida.aberto_em,
        AbertoEm::Sobrestado(vec![sobrestada("Unidade1", "Unidade1")])
    );
    assert_eq!(lida.documentos.len(), 1);
}

/// UFRJ e GO ao vivo (10/11/2026): o sobrestamento traz a atribuição.
#[test]
fn sobrestado_com_atribuicao_e_lido() {
    let u1 = ancora("Administração Central", "ADCENTRAL");
    let pessoa = ancora("Usuário teste 1", "servidor1");
    let caso = HtmlServido::de_teste(&format!(
        "Processo sobrestado na unidade {u1} (atribuído para {pessoa}).<br />"
    ));
    assert_eq!(
        ler_sobrestamento(&caso),
        Some(vec![sobrestada("ADCENTRAL", "Administração Central")])
    );
}

#[test]
fn sobrestamento_fora_da_forma_observada_nao_e_lido() {
    let u1 = ancora("Unidade1", "Unidade1");
    let u2 = ancora("Unidade2", "Unidade2");
    let casos = [
        "Processo sobrestado.<br />".to_owned(),
        format!("Processo sobrestado na unidade {u1}<br />"),
        format!("Processo sobrestado na unidade {u1}."),
        format!("{}x", sobrestado(&u1)),
        format!("Processo sobrestado nas unidades:<br />{u1}<br />{u2}<br /><br />"),
        format!("Processo sobrestado na unidade {u1}{u2}.<br />"),
        sobrestado("<a alt=\"A\" title=\"B\" class=\"ancoraSigla\">U</a>"),
        somente(&u1),
    ];
    for caso in casos {
        assert_eq!(
            ler_sobrestamento(&HtmlServido::de_teste(&caso)),
            None,
            "{caso}"
        );
        assert!(
            !matches!(ler_aberto_em(&com_html(&caso)), AbertoEm::Sobrestado(_)),
            "{caso}"
        );
    }
}
