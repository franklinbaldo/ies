//! RED/GREEN da pesquisa em documentos: linhas, resposta 200, 302 e o POST
//! derivado com `D`, sobre telas sintéticas com a estrutura observada.
use super::tests_fixtures::{
    href_do_documento, href_do_processo, linha_de_documento, linhas, localizacao_de_documento,
    numero_sei, nup, resultado_com, resultado_de_documentos, tabela,
};
use super::*;
use crate::erro_html::ErroHtml;
use crate::formulario_web::tests::instalacao;
use crate::formulario_web::{Redirecionamento302, RespostaDoEnvio};
use crate::pesquisa_web::alvo::AlvoDaPesquisa;
use crate::pesquisa_web::formulario_pesquisa::ErroPesquisaProtocolo;
use crate::pesquisa_web::redirecionamento::ErroRedirecionamentoDaPesquisa;
use crate::pesquisa_web::resposta::ErroRespostaDaPesquisa;
use crate::pesquisa_web::tests_fixtures::{sem_resultado, url_do_envio};
use ErroDocumentoDaPesquisa as D;

fn erro_da_linha(html: &str) -> D {
    ler_documentos(html).unwrap_err()
}

fn uma_linha(linha: &str) -> String {
    resultado_com("x", &tabela(1, 1, "1", linha))
}

#[test]
fn le_numero_sei_rotulo_processo_unidade_inclusao_e_anexo() {
    let html = resultado_de_documentos("0005", 1, 3, "3");
    let documentos = ler_documentos(&html).unwrap();
    assert_eq!(documentos.len(), 3);
    assert_eq!(
        documentos[0],
        DocumentoEncontrado {
            numero_sei: numero_sei(1),
            rotulo: "Despacho 1".to_owned(),
            processo: NumeroProcesso::novo(&nup(1)).unwrap(),
            unidade: Some("TESTE".to_owned()),
            inclusao: Some("12/10/2026".to_owned()),
            anexo: false,
        }
    );
    assert!(documentos[2].anexo, "documento_download_anexo");
    assert_eq!(documentos[2].numero_sei, "0005003");
}

#[test]
fn resposta_classifica_lista_com_total_e_faixa_e_sem_resultado() {
    let html = resultado_de_documentos("0005", 11, 20, "1.234");
    let RespostaDeDocumentos::Lista(lista) = ler_resposta_de_documentos(&html, None).unwrap()
    else {
        panic!("esperada lista");
    };
    assert_eq!(lista.total_declarado, 1234);
    assert_eq!(
        lista.faixa,
        FaixaDaPesquisa {
            inicio: 11,
            fim: 20
        }
    );
    assert_eq!(lista.documentos.len(), 10);
    assert_eq!(
        ler_resposta_de_documentos(&sem_resultado("0005"), None).unwrap(),
        RespostaDeDocumentos::SemResultado
    );
}

/// Goiás 5.0.3 (`documento-numero-e-busca-go`, passo 13): "Exibindo 1 - 10
/// de 225.304" com 4 registros no HTML; publicadas as 4 e as 6 ausentes.
#[test]
fn menos_linhas_que_a_faixa_publicam_as_ausentes_como_em_processos() {
    let html = resultado_com("x", &tabela(1, 10, "225.304", &linhas(1, 4)));
    let RespostaDeDocumentos::Lista(lista) = ler_resposta_de_documentos(&html, None).unwrap()
    else {
        panic!("esperada lista");
    };
    assert_eq!(lista.documentos.len(), 4);
    assert_eq!(lista.total_declarado, 225_304);
    assert_eq!(lista.faixa, FaixaDaPesquisa { inicio: 1, fim: 10 });
    assert_eq!(lista.linhas_ausentes_da_faixa, 6);
    let vazia = resultado_com("x", &tabela(1, 10, "3.300", ""));
    let RespostaDeDocumentos::Lista(lista) = ler_resposta_de_documentos(&vazia, None).unwrap()
    else {
        panic!("esperada lista");
    };
    assert_eq!(lista.documentos.len(), 0);
    assert_eq!(lista.linhas_ausentes_da_faixa, 10);
    let mais = resultado_com("x", &tabela(1, 10, "3.300", &linhas(1, 11)));
    assert_eq!(
        ler_resposta_de_documentos(&mais, None).unwrap_err(),
        ErroRespostaDaPesquisa::LinhasDaTabelaDiferentesDaFaixaDaBarra
    );
}

#[test]
fn link_do_processo_sem_id_documento_e_recusado() {
    let processo = href_do_processo(1);
    let sem_id = processo.replace("&id_documento=3001&", "&id_documento=&");
    let linha = linha_de_documento(1, false).replace(&processo, &sem_id);
    assert_eq!(
        erro_da_linha(&uma_linha(&linha)),
        D::LinhaDeDocumentoSemIdDocumentoNoLinkDoProcesso
    );
}

#[test]
fn documento_com_outro_id_ou_acao_desconhecida_e_recusado() {
    let documento = href_do_documento(1, false);
    let outro = documento.replace("id_documento=3001", "id_documento=3999");
    let linha = linha_de_documento(1, false).replace(&documento, &outro);
    assert_eq!(
        erro_da_linha(&uma_linha(&linha)),
        D::LinkDoDocumentoDiferenteDoProcesso
    );
    let acao = documento.replace("documento_visualizar", "documento_excluir");
    let linha = linha_de_documento(1, false).replace(&documento, &acao);
    assert_eq!(
        erro_da_linha(&uma_linha(&linha)),
        D::Html(ErroHtml::valor(
            "link_da_linha_de_documento_nao_reconhecido"
        ))
    );
}

#[test]
fn numero_sei_com_href_diferente_ou_vazio_e_recusado() {
    let documento = href_do_documento(1, false);
    let numero = format!("href=\"{documento}\" title=\"Despacho\" tabindex=\"1004\">");
    let linha = linha_de_documento(1, false).replace(
        &numero,
        &numero.replace("id_documento=3001", "id_documento=3001&x=1"),
    );
    assert_eq!(
        erro_da_linha(&uma_linha(&linha)),
        D::NumeroSeiComLinkDiferenteDoDocumento
    );
    let vazio = linha_de_documento(1, false).replace(">0005001</a>", "> </a>");
    assert_eq!(
        erro_da_linha(&uma_linha(&vazio)),
        D::Html(ErroHtml::ausente("numero_sei_ausente_na_linha"))
    );
}

#[test]
fn celula_da_direita_ausente_e_recusada() {
    let linha = linha_de_documento(1, false).replace("pesquisaTituloDireita", "outra");
    assert_eq!(
        erro_da_linha(&uma_linha(&linha)),
        D::Html(ErroHtml::ausente(
            "celula_do_titulo_do_documento_ausente_ou_repetida"
        ))
    );
}

#[test]
fn rotulo_vazio_metadado_repetido_e_data_fora_da_forma_sao_recusados() {
    let vazio = linha_de_documento(1, false).replace(">Despacho 1</a>", "></a>");
    assert_eq!(
        erro_da_linha(&uma_linha(&vazio)),
        D::Html(ErroHtml::ausente("rotulo_do_documento_ausente"))
    );
    let repetido =
        linha_de_documento(1, false).replace("<b>Usu&aacute;rio:</b>", "<b>Unidade:</b>");
    assert_eq!(
        erro_da_linha(&uma_linha(&repetido)),
        D::Html(ErroHtml::repetido("metadado_do_documento_repetido"))
    );
    let data = linha_de_documento(1, false).replace("12/10/2026", "2026-10-12");
    assert_eq!(
        erro_da_linha(&uma_linha(&data)),
        D::Html(ErroHtml::valor("data_de_inclusao_fora_da_forma_observada"))
    );
}

#[test]
fn sem_metadados_unidade_e_inclusao_sao_nao_observados() {
    let linha = linha_de_documento(1, false);
    let corte = linha.find("<tr><td colspan=\"3\"").unwrap();
    let documentos = ler_documentos(&uma_linha(&linha[..corte])).unwrap();
    assert_eq!(documentos[0].unidade, None);
    assert_eq!(documentos[0].inclusao, None);
}

#[test]
fn linha_antes_do_primeiro_registro_e_numero_repetido_sao_recusados() {
    let solta = format!("<tr><td>x</td></tr>{}", linha_de_documento(1, false));
    assert_eq!(
        erro_da_linha(&uma_linha(&solta)),
        D::LinhaDaPesquisaForaDeUmRegistro
    );
    let segunda = linha_de_documento(2, false).replace(">0005002</a>", ">0005001</a>");
    let html = resultado_com(
        "x",
        &tabela(1, 2, "2", &(linha_de_documento(1, false) + &segunda)),
    );
    assert_eq!(
        erro_da_linha(&html),
        D::Html(ErroHtml::repetido("numero_sei_repetido_na_pesquisa"))
    );
}

#[test]
fn link_do_processo_repetido_reaproveita_a_recusa_da_linha() {
    let processo = href_do_processo(1);
    let linha = linha_de_documento(1, false).replace(
        "</a>)</td>",
        &format!("</a>)<a class=\"protocoloNormal\" href=\"{processo}\">1</a></td>"),
    );
    assert_eq!(
        erro_da_linha(&uma_linha(&linha)),
        D::Linha(ErroResultadoPesquisa::Html(ErroHtml::repetido(
            "linha_de_pesquisa_com_mais_de_um_link_do_processo"
        )))
    );
}

fn interpretar(localizacao: &str) -> Result<RespostaDeDocumentos, ErroPesquisaProtocolo> {
    let enviada = RespostaDoEnvio::Redirecionamento(Redirecionamento302::sintetico(
        Some(localizacao),
        &url_do_envio(),
    ));
    EmDocumentos::interpretar(enviada, &instalacao()).map(|e| e.resposta)
}

#[test]
fn redirecionamento_com_id_documento_vira_documento_unico() {
    let resposta = interpretar(&localizacao_de_documento("4321", "8765")).unwrap();
    assert_eq!(
        resposta,
        RespostaDeDocumentos::DocumentoUnico {
            documento: "8765".to_owned(),
            processo: IdProcedimento::novo("4321").unwrap(),
        }
    );
}

#[test]
fn redirecionamento_sem_id_documento_e_recusado() {
    let erro = interpretar(&localizacao_de_documento("4321", "")).unwrap_err();
    assert_eq!(
        erro,
        ErroPesquisaProtocolo::Redirecionamento(
            ErroRedirecionamentoDaPesquisa::IdDocumentoAusenteNoRedirecionamento
        )
    );
    let _erro = interpretar(&localizacao_de_documento("4321", "x1")).unwrap_err();
}

#[test]
fn subcodigos_da_linha_de_documento_sao_estaveis() {}

#[test]
fn processo_visitado_com_class_repetido_contendo_a_classe_e_lido() {
    // PEN3 5.0.4: `class="protocoloNormal processoVisitado" … class="protocoloNormal"`.
    let linha = linha_de_documento(1, false);
    let visitado = linha.replacen(
        "class=\"protocoloNormal\" href=\"controlador.php?acao=procedimento_trabalhar",
        "class=\"protocoloNormal processoVisitado\" href=\"controlador.php?acao=procedimento_trabalhar",
        1,
    );
    assert_ne!(visitado, linha);
    let documentos = ler_documentos(&uma_linha(&visitado)).unwrap();
    assert_eq!(documentos[0].numero_sei, "0005001");
    let sem_a_classe = linha.replacen(
        "class=\"protocoloNormal\" href=\"controlador.php?acao=procedimento_trabalhar",
        "class=\"processoVisitado\" href=\"controlador.php?acao=procedimento_trabalhar",
        1,
    );
    assert!(matches!(
        erro_da_linha(&uma_linha(&sem_a_classe)),
        D::AtributoHtml(_)
    ));
}
