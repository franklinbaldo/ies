//! RED/GREEN das pastas no fluxo: sequência exata de requisições (frio com
//! e sem pastas fechadas, quente pelo link de "Abrir todas as Pastas"),
//! aprendizado no índice e recusas tipadas. Árvores sintéticas.
use super::tests_apoio::*;
use super::*;
use crate::acao_do_sei::AcaoDoSei;
use crate::arvore_processo::ErroArvore;
use crate::arvore_processo::tests::NUP;
use crate::arvore_processo::tests_pastas::{
    No, PASTA1, PASTA2, acoes_das_pastas, arvore_aberta, arvore_aberta_com, arvore_em_pastas,
    arvore_fechada, declaracoes, url_abrir,
};

const OBJETO_ABRIR: &str = "10:abrir_pastas";

fn url_abrir_absoluta() -> String {
    absoluta(&url_abrir(ID))
}

fn link(amb: &Ambiente, objeto: &str) -> Option<String> {
    let g = geracao();
    amb.indice
        .link(
            &instalacao(),
            chave(&g, AcaoDoSei::ProcedimentoVisualizar, objeto),
        )
        .map(|u| u.url().to_owned())
}

fn guardar(amb: &Ambiente, objeto: &str, url: &str) {
    let (inst, g) = (instalacao(), geracao());
    amb.indice.registrar_processo(&inst, NUP, ID);
    amb.indice
        .registrar_link(
            &inst,
            chave(&g, AcaoDoSei::ProcedimentoVisualizar, objeto),
            url,
        )
        .unwrap();
}

fn pelo_controle(arvores: Vec<Resposta>) -> Vec<Resposta> {
    let mut respostas = vec![ok(controle(&[(ID, NUP)])), ok(trabalhar(ID))];
    respostas.extend(arvores);
    respostas
}

fn pedidos_pelo_controle(arvores: &[String]) -> Vec<Pedido> {
    let mut pedidos = vec![
        get(&url_controle()),
        get(&absoluta(&trabalhar_controle(ID))),
    ];
    pedidos.extend(arvores.iter().map(|u| get(u)));
    pedidos
}

fn ids(r: &ProcessoAberto) -> Vec<(&str, Option<&str>)> {
    r.arvore
        .documentos
        .iter()
        .map(|d| (d.id.as_str(), d.pasta.as_deref()))
        .collect()
}

const TODOS: [(&str, Option<&str>); 4] = [
    ("21", Some("PASTA1")),
    ("22", Some("PASTA1")),
    ("31", Some("PASTA2")),
    ("32", Some("PASTA2")),
];

#[test]
fn frio_com_pasta_fechada_abre_todas_com_um_get_e_aprende_o_link() {
    let amb = ambiente();
    let (r, pedidos) = rodar(
        &amb,
        None,
        pelo_controle(vec![ok(arvore_fechada()), ok(arvore_aberta())]),
    );
    let r = r.unwrap();
    assert_eq!(r.requisicoes, 4);
    assert_eq!(
        pedidos,
        pedidos_pelo_controle(&[url_arvore(ID), url_abrir_absoluta()])
    );
    assert_eq!(ids(&r), TODOS);
    assert!(!r.arvore.pastas_nao_carregadas);
    assert_eq!(r.arvore.pastas.len(), 2);
    // O link da árvore simples e o de abrir pastas, em chaves distintas.
    assert_eq!(link(&amb, ID), Some(url_arvore(ID)));
    assert_eq!(link(&amb, OBJETO_ABRIR), Some(url_abrir_absoluta()));
    assert!(!format!("{r:?}").contains(&"f".repeat(64)));
}

#[test]
fn frio_sem_pasta_fechada_nao_faz_requisicao_a_mais() {
    let amb = ambiente();
    let (r, pedidos) = rodar(&amb, None, pelo_controle(vec![ok(arvore_aberta())]));
    let r = r.unwrap();
    assert_eq!(r.requisicoes, 3);
    assert_eq!(pedidos, pedidos_pelo_controle(&[url_arvore(ID)]));
    assert_eq!(ids(&r), TODOS);
    assert_eq!(link(&amb, OBJETO_ABRIR), None);
}

#[test]
fn quente_vai_direto_a_arvore_aberta() {
    let amb = ambiente();
    guardar(&amb, ID, &url_arvore(ID));
    guardar(&amb, OBJETO_ABRIR, &url_abrir_absoluta());
    let (r, pedidos) = rodar(&amb, None, vec![ok(arvore_aberta())]);
    let r = r.unwrap();
    assert_eq!(r.resolvido_por, ResolvidoPor::IndiceLink);
    assert_eq!(r.requisicoes, 1);
    assert_eq!(pedidos, [get(&url_abrir_absoluta())]);
    assert_eq!(ids(&r), TODOS);
}

#[test]
fn quente_com_link_de_abrir_vencido_esquece_e_segue_pela_arvore_guardada() {
    let amb = ambiente();
    guardar(&amb, ID, &url_arvore(ID));
    guardar(&amb, OBJETO_ABRIR, &url_abrir_absoluta());
    let (r, pedidos) = rodar(
        &amb,
        None,
        vec![r302(LOGIN), ok(arvore_fechada()), ok(arvore_aberta())],
    );
    let r = r.unwrap();
    assert_eq!(r.resolvido_por, ResolvidoPor::IndiceLink);
    assert_eq!(r.requisicoes, 3);
    assert_eq!(
        pedidos,
        [
            get(&url_abrir_absoluta()),
            get(&url_arvore(ID)),
            get(&url_abrir_absoluta()),
        ]
    );
    assert_eq!(ids(&r), TODOS);
    // Reaprendido da árvore que acabou de ser lida.
    assert_eq!(link(&amb, OBJETO_ABRIR), Some(url_abrir_absoluta()));
}

#[test]
fn quente_pela_arvore_guardada_com_pasta_fechada_custa_dois() {
    let amb = ambiente();
    guardar(&amb, ID, &url_arvore(ID));
    let (r, pedidos) = rodar(&amb, None, vec![ok(arvore_fechada()), ok(arvore_aberta())]);
    let r = r.unwrap();
    assert_eq!(r.requisicoes, 2);
    assert_eq!(pedidos, [get(&url_arvore(ID)), get(&url_abrir_absoluta())]);
    assert_eq!(link(&amb, OBJETO_ABRIR), Some(url_abrir_absoluta()));
}

#[test]
fn pasta_fechada_sem_a_acao_de_abrir_e_recusa_sem_requisicao_a_mais() {
    let amb = ambiente();
    let sem_acao = arvore_fechada().replace("\"ABRIR_PASTAS\"", "\"OUTRA_ACAO\"");
    let (r, pedidos) = rodar(&amb, None, pelo_controle(vec![ok(sem_acao)]));
    assert_eq!(
        r.unwrap_err(),
        super::ErroProcessoObter::Arvore(ErroArvore::PastasNaoCarregadasSemAcaoDeAbrir)
    );
    assert_eq!(pedidos.len(), 3);
}

#[test]
fn arvore_ainda_fechada_depois_de_abrir_e_recusa() {
    let amb = ambiente();
    let (r, _) = rodar(
        &amb,
        None,
        pelo_controle(vec![ok(arvore_fechada()), ok(arvore_fechada())]),
    );
    assert_eq!(
        r.unwrap_err(),
        super::ErroProcessoObter::Arvore(ErroArvore::PastasNaoCarregadasAposAbrirPastas)
    );
    assert_eq!(link(&amb, OBJETO_ABRIR), None);
}

#[test]
fn arvore_aberta_que_nao_confere_nao_publica_lista_parcial() {
    let amb = ambiente();
    let faltando = arvore_aberta_com(&[
        No::Documento("21", "PASTA1".into(), Some("900")),
        No::Documento("22", "PASTA1".into(), Some("901")),
        No::Documento("31", "PASTA2".into(), Some("902")),
    ]);
    let (r, _) = rodar(
        &amb,
        None,
        pelo_controle(vec![ok(arvore_fechada()), ok(faltando)]),
    );
    assert_eq!(
        r.unwrap_err(),
        super::ErroProcessoObter::Arvore(ErroArvore::ProtocoloDeclaradoAusenteDaPasta)
    );
    assert_eq!(link(&amb, OBJETO_ABRIR), None);
}

#[test]
fn abrir_pastas_que_responde_login_ou_outro_processo_e_erro() {
    let amb = ambiente();
    let (r, _) = rodar(
        &amb,
        None,
        pelo_controle(vec![ok(arvore_fechada()), r302(LOGIN)]),
    );
    assert_eq!(
        r.unwrap_err(),
        super::ErroProcessoObter::SessaoEncerrada(crate::erro_publico::SessaoEncerrada)
    );
    let amb = ambiente();
    let outra = arvore_aberta().replace(NUP, "00002.000002/2026-02");
    let (r, _) = rodar(
        &amb,
        None,
        pelo_controle(vec![ok(arvore_fechada()), ok(outra)]),
    );
    assert_eq!(
        r.unwrap_err(),
        super::ErroProcessoObter::Arvore(ErroArvore::IdentidadeDoProcessoDiferenteDaPedida)
    );
}

#[test]
fn todas_as_pastas_fechadas_custam_um_get() {
    let amb = ambiente();
    let fechadas = arvore_em_pastas(
        &declaracoes(&[(1, &PASTA1), (2, &PASTA2)]),
        &[
            No::Processo,
            No::Pasta(1, false),
            No::Aguarde(1),
            No::Pasta(2, false),
            No::Aguarde(2),
        ],
        &acoes_das_pastas(&url_abrir(ID)),
    );
    let (r, pedidos) = rodar(
        &amb,
        None,
        pelo_controle(vec![ok(fechadas), ok(arvore_aberta())]),
    );
    assert_eq!(r.unwrap().requisicoes, 4);
    assert_eq!(pedidos.len(), 4);
}

#[test]
fn arvore_aberta_com_barras_vazias_sai_por_contagem() {
    let amb = ambiente();
    let vazias = arvore_aberta_com(&[
        No::Documento("21", "PASTA1".into(), None),
        No::Documento("22", "PASTA1".into(), None),
        No::Documento("31", "PASTA2".into(), None),
        No::Documento("32", "PASTA2".into(), None),
    ]);
    let (r, pedidos) = rodar(
        &amb,
        None,
        pelo_controle(vec![ok(arvore_fechada()), ok(vazias)]),
    );
    let r = r.unwrap();
    assert_eq!(pedidos.len(), 4);
    assert_eq!(ids(&r), TODOS);
    assert!(r.arvore.pastas.iter().all(|p| p.verificacao
        == Some(crate::arvore_processo::VerificacaoDaPasta::PorContagem)));
}
