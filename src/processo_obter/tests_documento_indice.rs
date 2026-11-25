//! RED/GREEN de `documento DOC` pelo índice e por `--processo NUP` (Refs
//! #60, fatia 2): sequência exata das requisições de cada caminho, o que o
//! índice aprende e esquece, e a entrada por id com documento. Telas
//! SINTÉTICAS com a forma das cassettes `documento-numero-e-busca-*` e
//! `processo-em-pastas-unb`; hosts, ids, números e hashes inventados.
use super::tests_apoio::*;
use super::*;
use crate::acao_do_sei::AcaoDoSei;
use crate::arvore_processo::tests::{NUP, arvore, no_documento, no_processo};
use crate::arvore_processo::tests_conteudo::{com_src, src_da_arvore_do_documento, src_visualizar};
use crate::arvore_processo::tests_pastas::{arvore_aberta, arvore_fechada, url_abrir};
use crate::arvore_processo::tests_tela::tela_do_processo;
use crate::documento::NumeroDocumento;
use crate::processo::NumeroProcesso;
use crate::transporte::sessao::CookieDaSessao;

const DOC: &str = "0000101";
const ID_DOC: &str = "21";
const DOC2: &str = "0000102";
const ID_DOC2: &str = "22";
const CONTEXTO_SINTETICO: &str = "infra_sistema=100000100&infra_unidade_atual=110000001";

/// Árvore do processo `ID` com os dois documentos e o `.src` de cada um.
fn arvore_dos_dois() -> String {
    let html = arvore(&[
        no_processo(0, ID, NUP),
        no_documento(1, ID_DOC, ID, DOC),
        no_documento(2, ID_DOC2, ID, DOC2),
    ]);
    com_src(
        &com_src(&html, 1, &src_visualizar(ID_DOC)),
        2,
        &src_visualizar(ID_DOC2),
    )
}

fn conteudo(numero: &str) -> String {
    format!(
        "<html><head><title>SEI/EXEMPLO - {numero} - Despacho</title></head>\
         <body><p>Texto de {numero}.</p></body></html>"
    )
}

fn url_conteudo(id: &str) -> String {
    absoluta(&src_visualizar(id))
}

fn tela_do_documento(documento: &str) -> String {
    tela_do_processo(UNIDADE, &[src_da_arvore_do_documento(ID, documento)])
        .replace("</body>", &format!("{}</body>", formulario_rapido()))
}

fn url_entrada_doc(id: &str, documento: &str) -> String {
    format!("{}&id_documento={documento}", url_entrada(id))
}

/// `Location` da entrada com documento aceita: a mesma ação assinada, os
/// dois ids, sem `acao_origem`.
fn trabalhar_entrada_doc(id: &str, documento: &str) -> String {
    format!(
        "{ORIGEM}/controlador.php?acao=procedimento_trabalhar&id_procedimento={id}&id_documento={documento}&infra_hash={}&{CONTEXTO_SINTETICO}",
        hash('8')
    )
}

fn rodar_doc(
    amb: &Ambiente,
    numero: &str,
    processo: Option<&str>,
    respostas: Vec<Resposta>,
) -> (Result<DocumentoObtido, ErroDocumentoObter>, Vec<Pedido>) {
    let url = url_controle();
    let sessao = SessaoDoProcesso {
        url_do_controle: &url,
        origem: ORIGEM,
        cookie: &CookieDaSessao::ler(&amb.cookie),
    };
    let numero = NumeroDocumento::novo(numero).unwrap();
    let nup = processo.map(|p| NumeroProcesso::novo(p).unwrap());
    let pedido = PedidoDoDocumento {
        numero: &numero,
        unidade: None,
        processo: nup.as_ref(),
        anexo: PedidoDoAnexo::default(),
    };
    let mut roteiro = Roteiro::novo(respostas);
    let r = obter_documento(&sessao, &pedido, &amb.indice, &mut roteiro);
    assert_eq!(roteiro.respostas_restantes(), 0, "respostas não consumidas");
    (r, roteiro.pedidos)
}

/// A fatia 1 pelo Controle (5 requisições): aprende o processo, o link da
/// árvore, o `action` da pesquisa e os dois documentos.
fn primeira_leitura(amb: &Ambiente) {
    let (r, pedidos) = rodar_doc(
        amb,
        DOC,
        None,
        vec![
            ok(controle(&[("7", "OUTRO-1")])),
            r302(&trabalhar_rapida_documento(ID_DOC)),
            ok(tela_do_documento(ID_DOC)),
            ok(arvore_dos_dois()),
            ok(conteudo(DOC)),
        ],
    );
    assert_eq!(r.unwrap().requisicoes, 5);
    assert_eq!(pedidos.len(), 5);
}

fn semear(amb: &Ambiente, documentos: &[(&str, &str)]) {
    let inst = instalacao();
    amb.indice.registrar_processo(&inst, NUP, ID);
    amb.indice.registrar_documentos(&inst, ID, documentos);
}

fn linha(numero: &str, processo: &str, documento: &str) -> [String; 4] {
    [ORIGEM, numero, processo, documento].map(str::to_owned)
}

#[test]
fn toda_arvore_lida_ensina_os_documentos_ao_indice() {
    let amb = ambiente();
    primeira_leitura(&amb);
    assert_eq!(
        amb.indice.documentos(),
        [linha(DOC, ID, ID_DOC), linha(DOC2, ID, ID_DOC2)]
    );
    // `processo NUP` também ensina.
    let amb = ambiente();
    let (r, _) = rodar(
        &amb,
        None,
        vec![
            ok(controle(&[(ID, NUP)])),
            ok(trabalhar(ID)),
            ok(arvore_de(ID, NUP)),
        ],
    );
    r.unwrap();
    assert_eq!(amb.indice.documentos(), [linha("0000101", ID, "21")]);
}

#[test]
fn outro_documento_do_mesmo_processo_pelo_link_da_arvore_em_duas() {
    let amb = ambiente();
    primeira_leitura(&amb);
    let arvore_guardada = absoluta(&src_da_arvore_do_documento(ID, ID_DOC));
    let (r, pedidos) = rodar_doc(
        &amb,
        DOC2,
        None,
        vec![ok(arvore_dos_dois()), ok(conteudo(DOC2))],
    );
    let d = r.unwrap();
    assert_eq!(
        pedidos,
        [get(&arvore_guardada), get(&url_conteudo(ID_DOC2))]
    );
    assert_eq!(d.requisicoes, 2);
    assert_eq!(d.resolvido_por, ResolvidoPor::IndiceLink);
    assert_eq!(
        (d.id_documento.as_str(), d.processo.nup.as_str()),
        (ID_DOC2, NUP)
    );
    assert_eq!(d.unidade, None, "a árvore não publica a unidade");
    assert_eq!(d.texto, "Texto de 0000102.");
}

#[test]
fn link_de_trabalho_guardado_em_tres() {
    let amb = ambiente();
    semear(&amb, &[(DOC, ID_DOC)]);
    let (inst, g) = (instalacao(), geracao());
    let trabalho = absoluta(&trabalhar_controle(ID));
    amb.indice
        .registrar_link(
            &inst,
            chave(&g, AcaoDoSei::ProcedimentoTrabalhar, ID),
            &trabalho,
        )
        .unwrap();
    let (r, pedidos) = rodar_doc(
        &amb,
        DOC,
        None,
        vec![ok(trabalhar(ID)), ok(arvore_dos_dois()), ok(conteudo(DOC))],
    );
    let d = r.unwrap();
    assert_eq!(
        pedidos,
        [
            get(&trabalho),
            get(&url_arvore(ID)),
            get(&url_conteudo(ID_DOC))
        ]
    );
    assert_eq!(d.resolvido_por, ResolvidoPor::IndiceTrabalhar);
    assert_eq!(d.unidade.as_deref(), Some(UNIDADE));
}

#[test]
fn entrada_por_id_com_documento_em_quatro_e_aceite_registrado() {
    let amb = ambiente();
    semear(&amb, &[(DOC, ID_DOC)]);
    let (r, pedidos) = rodar_doc(
        &amb,
        DOC,
        None,
        vec![
            r302(&trabalhar_entrada_doc(ID, ID_DOC)),
            ok(tela_do_documento(ID_DOC)),
            ok(arvore_dos_dois()),
            ok(conteudo(DOC)),
        ],
    );
    let d = r.unwrap();
    assert_eq!(
        pedidos,
        [
            get(&url_entrada_doc(ID, ID_DOC)),
            get(&trabalhar_entrada_doc(ID, ID_DOC)),
            get(&absoluta(&src_da_arvore_do_documento(ID, ID_DOC))),
            get(&url_conteudo(ID_DOC)),
        ]
    );
    assert_eq!(d.resolvido_por, ResolvidoPor::IndiceEntradaDocumento);
    assert_eq!(d.requisicoes, 4);
    let inst = instalacao();
    assert_eq!(
        amb.indice.entrada_por_id_com_documento(&inst),
        Some(crate::indice_local::AceiteDaEntrada::Aceita)
    );
    assert!(!format!("{d:?}").contains(&hash('8')));
}

#[test]
fn recusa_da_entrada_com_documento_para_e_a_proxima_vai_pela_pesquisa() {
    let amb = ambiente();
    semear(&amb, &[(DOC, ID_DOC)]);
    let (r, pedidos) = rodar_doc(&amb, DOC, None, vec![r302(LOGIN_RECUSA)]);
    assert_eq!(
        r.unwrap_err(),
        ErroDocumentoObter::Processo(ErroProcessoObter::EntradaPorIdRecusada)
    );
    assert_eq!(pedidos, [get(&url_entrada_doc(ID, ID_DOC))]);
    let inst = instalacao();
    assert_eq!(
        amb.indice.entrada_por_id_com_documento(&inst),
        Some(crate::indice_local::AceiteDaEntrada::Recusada)
    );
    // Recusada a forma com documento, nenhuma outra URL é montada.
    let (r, pedidos) = rodar_doc(
        &amb,
        DOC,
        None,
        vec![
            ok(controle(&[])),
            r302(&trabalhar_rapida_documento(ID_DOC)),
            ok(tela_do_documento(ID_DOC)),
            ok(arvore_dos_dois()),
            ok(conteudo(DOC)),
        ],
    );
    assert_eq!(r.unwrap().resolvido_por, ResolvidoPor::PesquisaRapida);
    assert_eq!(pedidos[0], get(&url_controle()));
}

#[test]
fn entrada_que_enderecou_outro_documento_esquece_e_cai_na_pesquisa_uma_vez() {
    let amb = ambiente();
    semear(&amb, &[(DOC, ID_DOC)]);
    let (r, pedidos) = rodar_doc(
        &amb,
        DOC,
        None,
        vec![
            r302(&trabalhar_entrada_doc(ID, "99")),
            ok(controle(&[])),
            r302(&trabalhar_rapida_documento(ID_DOC)),
            ok(tela_do_documento(ID_DOC)),
            ok(arvore_dos_dois()),
            ok(conteudo(DOC)),
        ],
    );
    let d = r.unwrap();
    assert_eq!(d.requisicoes, 6);
    assert_eq!(pedidos[1], get(&url_controle()));
    assert_eq!(d.resolvido_por, ResolvidoPor::PesquisaRapida);
}

#[test]
fn indice_desmentido_pela_arvore_e_esquecido_e_cai_na_pesquisa() {
    let amb = ambiente();
    primeira_leitura(&amb);
    // O índice diz que `0000103` é do processo `ID`; a árvore não o tem.
    amb.indice
        .registrar_documentos(&instalacao(), ID, &[("0000103", "23")]);
    let arvore_guardada = absoluta(&src_da_arvore_do_documento(ID, ID_DOC));
    let (_r, pedidos) = rodar_doc(
        &amb,
        "0000103",
        None,
        vec![
            ok(arvore_dos_dois()),
            ok("<html><body>pesquisa</body></html>".into()),
        ],
    );
    assert_eq!(
        pedidos,
        [
            get(&arvore_guardada),
            post(&absoluta(&action_rapida()), "txtPesquisaRapida=0000103"),
        ]
    );
    let numeros: Vec<String> = amb
        .indice
        .documentos()
        .into_iter()
        .map(|l| l[1].clone())
        .collect();
    assert_eq!(numeros, [DOC, DOC2], "o palpite desmentido foi esquecido");
}

#[test]
fn servidor_vence_documento_noutro_id_na_arvore_e_reaprendido() {
    let amb = ambiente();
    primeira_leitura(&amb);
    amb.indice
        .registrar_documentos(&instalacao(), ID, &[(DOC2, "29")]);
    let (r, pedidos) = rodar_doc(
        &amb,
        DOC2,
        None,
        vec![ok(arvore_dos_dois()), ok(conteudo(DOC2))],
    );
    assert_eq!(r.unwrap().id_documento, ID_DOC2);
    assert_eq!(pedidos.len(), 2);
    assert!(amb.indice.documentos().contains(&linha(DOC2, ID, ID_DOC2)));
}

#[test]
fn processo_informado_frio_pelo_controle_em_quatro() {
    let amb = ambiente();
    let (r, pedidos) = rodar_doc(
        &amb,
        DOC2,
        Some(NUP),
        vec![
            ok(controle(&[(ID, NUP)])),
            ok(trabalhar(ID)),
            ok(arvore_dos_dois()),
            ok(conteudo(DOC2)),
        ],
    );
    let d = r.unwrap();
    assert_eq!(
        pedidos,
        [
            get(&url_controle()),
            get(&absoluta(&trabalhar_controle(ID))),
            get(&url_arvore(ID)),
            get(&url_conteudo(ID_DOC2)),
        ]
    );
    assert_eq!(d.resolvido_por, ResolvidoPor::Caixa);
    assert_eq!(amb.indice.documentos().len(), 2);
}

#[test]
fn processo_informado_pelo_link_da_arvore_em_duas() {
    let amb = ambiente();
    semear(&amb, &[]);
    let (inst, g) = (instalacao(), geracao());
    amb.indice
        .registrar_link(
            &inst,
            chave(&g, AcaoDoSei::ProcedimentoVisualizar, ID),
            &url_arvore(ID),
        )
        .unwrap();
    let (r, pedidos) = rodar_doc(
        &amb,
        DOC,
        Some(NUP),
        vec![ok(arvore_dos_dois()), ok(conteudo(DOC))],
    );
    assert_eq!(r.unwrap().resolvido_por, ResolvidoPor::IndiceLink);
    assert_eq!(pedidos, [get(&url_arvore(ID)), get(&url_conteudo(ID_DOC))]);
    // Número que a árvore do processo não tem: recusa tipada, sem conteúdo.
    let (_r, pedidos) = rodar_doc(&amb, "0000999", Some(NUP), vec![ok(arvore_dos_dois())]);
    assert_eq!(pedidos.len(), 1);
}

/// A árvore de pastas sintética com `.src` assinado no documento `id`.
fn com_src_assinado(html: &str, id: &str) -> String {
    html.replace(
        &format!("'controlador.php?acao=documento_visualizar&id_documento={id}'"),
        &format!("'{}'", src_visualizar(id)),
    )
}

fn guardar_arvore(amb: &Ambiente) {
    semear(amb, &[]);
    let (inst, g) = (instalacao(), geracao());
    amb.indice
        .registrar_link(
            &inst,
            chave(&g, AcaoDoSei::ProcedimentoVisualizar, ID),
            &url_arvore(ID),
        )
        .unwrap();
}

#[test]
fn documento_em_pasta_fechada_abre_todas_as_pastas_uma_vez() {
    let amb = ambiente();
    guardar_arvore(&amb);
    // `0021` está na PASTA1, fechada (AGUARDE).
    let (r, pedidos) = rodar_doc(
        &amb,
        "0021",
        Some(NUP),
        vec![
            ok(arvore_fechada()),
            ok(com_src_assinado(&arvore_aberta(), "21")),
            ok(conteudo("0021")),
        ],
    );
    let d = r.unwrap();
    assert_eq!(
        pedidos,
        [
            get(&url_arvore(ID)),
            get(&absoluta(&url_abrir(ID))),
            get(&url_conteudo("21"))
        ]
    );
    assert_eq!(d.pasta.as_deref(), Some("PASTA1"));
    // `0031` está à vista na PASTA2 aberta: nenhuma requisição a mais.
    let amb = ambiente();
    guardar_arvore(&amb);
    let (r, pedidos) = rodar_doc(
        &amb,
        "0031",
        Some(NUP),
        vec![
            ok(com_src_assinado(&arvore_fechada(), "31")),
            ok(conteudo("0031")),
        ],
    );
    assert_eq!(r.unwrap().requisicoes, 2);
    assert_eq!(pedidos[1], get(&url_conteudo("31")));
}

#[test]
fn pesquisa_com_documento_em_pasta_fechada_abre_as_pastas() {
    let amb = ambiente();
    let (r, pedidos) = rodar_doc(
        &amb,
        "0021",
        None,
        vec![
            ok(controle(&[])),
            r302(&trabalhar_rapida_documento("21")),
            ok(tela_do_documento("21")),
            ok(arvore_fechada()),
            ok(com_src_assinado(&arvore_aberta(), "21")),
            ok(conteudo("0021")),
        ],
    );
    let d = r.unwrap();
    assert_eq!(d.requisicoes, 6);
    assert_eq!(pedidos[4], get(&absoluta(&url_abrir(ID))));
    assert_eq!(
        pedidos[1],
        post(&absoluta(&action_rapida()), "txtPesquisaRapida=0021")
    );
}

#[test]
fn bdd_60_05_numero_ambiguo_recusa_sem_contaminar_indice() {
    // Dado um processo com dois nós DOCUMENTO para o mesmo número SEI,
    // Quando executo ies documento DOC --processo NUP,
    // Então a CLI recusa a ambiguidade sem guardar um dos IDs como verdadeiro.
    let amb = ambiente();
    guardar_arvore(&amb);
    // Um palpite anterior não pode sobreviver à observação ambígua.
    amb.indice
        .registrar_documentos(&instalacao(), ID, &[(DOC, "99")]);
    let dois_ids_para_doc = arvore(&[
        no_processo(0, ID, NUP),
        no_documento(1, ID_DOC, ID, DOC),
        no_documento(2, "23", ID, DOC),
        no_documento(3, ID_DOC2, ID, DOC2),
    ]);
    let (resultado, pedidos) = rodar_doc(&amb, DOC, Some(NUP), vec![ok(dois_ids_para_doc)]);
    assert_eq!(
        resultado.unwrap_err(),
        ErroDocumentoObter::NumeroSeiRepetidoNaArvore
    );
    assert_eq!(pedidos, [get(&url_arvore(ID))], "nenhum GET de conteúdo");
    assert!(
        amb.indice
            .documento_por_numero(&instalacao(), DOC)
            .is_none(),
        "dois IDs observados não podem virar associação durável"
    );
    assert!(
        amb.indice
            .documento_por_numero(&instalacao(), DOC2)
            .is_some(),
        "outro número inequívoco deve continuar disponível"
    );
}
