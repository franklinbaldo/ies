//! BDD da prévia Web externa (#332), transporte roteirizado, nenhuma rede real.
//! Oráculo Todos `tests/test_documento_juntar_010.py` (`UnB` SEI 4).
use super::*;
use crate::processo_concluir::tests_apoio as a;
use crate::propriedades_processo::tests_apoio::acao_da_barra;

fn link_escolha() -> String {
    format!(
        "controlador.php?acao=documento_escolher_tipo&id_procedimento=10&infra_hash={}",
        "e".repeat(64)
    )
}
fn link_cadastro() -> String {
    format!(
        "controlador.php?acao=documento_receber&id_procedimento=10&infra_hash={}",
        "f".repeat(64)
    )
}
fn url(relativo: &str) -> String {
    format!("{}/{}", a::ORIGEM, relativo)
}
fn arvore() -> String {
    a::arvore(&[a::UNIDADE], &[acao_da_barra(&link_escolha())])
}
fn lista() -> String {
    let href = link_cadastro().replace('&', "&amp;");
    format!(
        r#"<html><table id="tblSeries"><tr><td><a href="{href}">Externo</a></td></tr></table></html>"#
    )
}
fn cadastro(flag: &str, escolhido: bool) -> String {
    let action = link_cadastro().replace('&', "&amp;");
    let selecionado = if escolhido {
        "selected=\"selected\""
    } else {
        ""
    };
    format!(
        r#"<html><body>
    <form id="frmDocumentoCadastro" method="post" action="{action}">
      <select id="selSerie" name="selSerie">
        <option value="null"> </option><option value="293" {selecionado}>Anexo</option>
      </select>
      <input type="hidden" id="hdnIdSerie" name="hdnIdSerie" value="{id}">
      <input type="hidden" id="hdnFlagDocumentoCadastro" name="hdnFlagDocumentoCadastro" value="{flag}">
      <input type="hidden" id="hdnAnexos" name="hdnAnexos" value="">
    </form><form id="frmAnexos"><input id="filArquivo" name="filArquivo" type="file"></form>
    <script>
    var arrExt = [];
    arrExt[0] = {{nome : "pdf", tamanho : 200}};
    objTabelaAnexos.adicionar([arr['nome_upload'],arr['nome'],arr['data_hora'],arr['tamanho'],infraFormatarTamanhoBytes(arr['tamanho'])]);
    objUpload = new infraUpload('frmAnexos','controlador.php?acao=documento_upload_anexo&infra_hash=FICTICIO');
    </script></body></html>"#,
        id = if escolhido { "293" } else { "" }
    )
}
fn executar(flag: &str) -> (Result<PreviaExterna, ErroPreviaExterna>, Vec<a::Pedido>) {
    let amb = a::ambiente();
    let mut respostas = a::abertura(arvore());
    respostas.push(a::ok(lista()));
    respostas.push(a::ok(cadastro(flag, false)));
    if flag == "1" {
        respostas.push(a::ok(cadastro("1", true)));
    }
    a::rodar_com(&amb, respostas, |sessao, nup, indice, rede| {
        let pdf = PdfLocal {
            nome: "prova.pdf".into(),
            tamanho_bytes: 68,
            sha256: "ficticio".into(),
        };
        executar_previa(sessao, nup, None, "Anexo", &pdf, indice, rede)
    })
}
#[test]
fn dado_sei4_quando_consulto_previa_entao_recarrega_tipo_sem_upload_nem_cadastro() {
    let (resultado, pedidos) = executar("1");
    let previa = resultado.unwrap_or_else(|e| panic!("prévia recusada: {e:?}"));
    assert_eq!(previa.id_tipo, "293");
    assert_eq!(previa.limite_pdf_bytes, 200 * 1024 * 1024);
    assert_eq!(previa.unidade, a::UNIDADE);
    let mut esperados = a::pedidos_da_abertura();
    esperados.push(a::get(&url(&link_escolha())));
    esperados.push(a::get(&url(&link_cadastro())));
    esperados.push(a::post(
        &url(&link_cadastro()),
        "selSerie=293&hdnIdSerie=293&hdnFlagDocumentoCadastro=1&hdnAnexos=",
    ));
    assert_eq!(
        pedidos, esperados,
        "um POST de recarga, zero upload e cadastro definitivo"
    );
}
#[test]
fn dado_flag_de_gravacao_quando_consulto_previa_entao_nao_envio_post() {
    let (resultado, pedidos) = executar("2");
    assert!(matches!(
        resultado,
        Err(ErroPreviaExterna::TipoNaoOferecido)
    ));
    let mut esperados = a::pedidos_da_abertura();
    esperados.push(a::get(&url(&link_escolha())));
    esperados.push(a::get(&url(&link_cadastro())));
    assert_eq!(pedidos, esperados, "flag 2 bloqueada antes de POST");
}

#[test]
fn dado_cadastro_minimo_quando_consulto_sei_entao_projeto_campo_visivel_no_resultado() {
    // BDD-332-02: o campo publicado vem do formulário recarregado.
    let (resultado, _) = executar("1");
    let previa = resultado.unwrap();
    assert_eq!(previa.campos_oferecidos, ["selSerie"]);
}

fn confirmar_com_upload_recusado(
    identidade: &str,
) -> (
    Result<
        super::super::fluxo_confirmar::InclusaoComprovada,
        super::super::fluxo_confirmar::ErroConfirmacaoExterna,
    >,
    Vec<a::Pedido>,
) {
    // Cenário BDD-332-04 previamente especificado na issue: nunca repetir upload.
    let amb = a::ambiente();
    let mut respostas = a::abertura(arvore());
    respostas.push(a::ok(lista()));
    respostas.push(a::ok(cadastro("1", false)));
    respostas.push(a::ok(cadastro("1", true).replace(
        "infra_hash=FICTICIO",
        &format!("infra_hash={}{identidade}", "b".repeat(64)),
    )));
    a::rodar_com(&amb, respostas, |sessao, nup, indice, rede| {
        let pdf = PdfLocal {
            nome: "prova.pdf".into(),
            tamanho_bytes: 68,
            sha256: "ficticio".into(),
        };
        super::super::fluxo_confirmar::executar_confirmacao_via(
            sessao,
            nup,
            None,
            "Anexo",
            &pdf,
            indice,
            rede,
            |_, _, _, _, _| {
                Err(super::super::recibo::FalhaUpload::Recusado {
                    motivo: "recusa sintetica".into(),
                })
            },
        )
    })
}

#[test]
fn bdd_332_04_dado_upload_recusado_quando_confirmo_entao_nao_cadastro() {
    for identidade in ["", "&id_procedimento=10"] {
        let (resultado, pedidos) = confirmar_com_upload_recusado(identidade);
        assert!(matches!(
            resultado,
            Err(
                super::super::fluxo_confirmar::ErroConfirmacaoExterna::Sequencia(
                    super::super::confirmacao::FalhaConfirmacao::UploadRecusado(_)
                )
            )
        ));
        assert_eq!(pedidos.len(), a::pedidos_da_abertura().len() + 3);
        assert!(
            pedidos
                .iter()
                .all(|p| !p.url.contains("documento_upload_anexo"))
        );
    }
}

#[test]
fn destino_de_outro_processo_ou_ambiguo_recusa_antes_do_upload() {
    for identidade in [
        "&id_procedimento=99",
        "&id_procedimento=10&id_procedimento=10",
    ] {
        let (resultado, pedidos) = confirmar_com_upload_recusado(identidade);
        assert!(matches!(
            resultado,
            Err(
                super::super::fluxo_confirmar::ErroConfirmacaoExterna::EnderecoDeUploadNaoObservado
            )
        ));
        assert_eq!(pedidos.len(), a::pedidos_da_abertura().len() + 3);
    }
}

#[test]
fn bdd_332_04_dado_recibo_e_cadastro_quando_releitura_nao_muda_entao_saida_3() {
    // Uma resposta 302 nao basta para atribuir a criacao ao nosso POST.
    let amb = a::ambiente();
    let mut respostas = a::abertura(arvore());
    respostas.push(a::ok(lista()));
    respostas.push(a::ok(cadastro("1", false)));
    let formulario = cadastro("1", true)
        .replace(
            "infra_hash=FICTICIO",
            &format!("infra_hash={}", "b".repeat(64)),
        )
        .replace(
            "</form><form id=\"frmAnexos\">",
            concat!(
                "<input type=\"text\" name=\"txtDataElaboracao\" value=\"\">",
                "<input type=\"radio\" name=\"rdoFormato\" value=\"N\" checked>",
                "<input type=\"radio\" name=\"rdoNivelAcesso\" value=\"0\" checked>",
                "</form><form id=\"frmAnexos\">",
            ),
        );
    respostas.push(a::ok(formulario));
    respostas.push(a::r302(
        "controlador.php?acao=arvore_visualizar&id_documento=42",
    ));
    respostas.push(a::ok(arvore()));
    let (resultado, pedidos) = a::rodar_com(&amb, respostas, |sessao, nup, indice, rede| {
        let pdf = PdfLocal {
            nome: "prova.pdf".into(),
            tamanho_bytes: 68,
            sha256: "ficticio".into(),
        };
        super::super::fluxo_confirmar::executar_confirmacao_via(
            sessao,
            nup,
            None,
            "Anexo",
            &pdf,
            indice,
            rede,
            |_, _, _, _, _| {
                Ok(super::super::recibo::ReciboUpload {
                    nome_upload: "temp-42".into(),
                    nome: "prova.pdf".into(),
                    tipo: "application/pdf".into(),
                    tamanho_bytes: 68,
                    data_hora: "11/10/2026 11:07:50".into(),
                })
            },
        )
    });
    assert!(matches!(
        resultado,
        Err(
            super::super::fluxo_confirmar::ErroConfirmacaoExterna::Sequencia(
                super::super::confirmacao::FalhaConfirmacao::NaoConfirmado
            )
        )
    ));
    assert_eq!(pedidos.len(), a::pedidos_da_abertura().len() + 5);
    let definitivos: Vec<_> = pedidos
        .iter()
        .filter(|p| {
            p.metodo == "POST"
                && p.corpo
                    .as_ref()
                    .is_some_and(|c| c.contains("hdnFlagDocumentoCadastro=2"))
        })
        .collect();
    assert_eq!(definitivos.len(), 1, "exatamente um cadastro definitivo");
    assert!(
        definitivos[0]
            .corpo
            .as_ref()
            .unwrap()
            .contains("hdnAnexos=temp-42")
    );
}

#[test]
fn bdd_332_04_formulario_sem_campos_de_cadastro_nao_deixa_upload_temporario() {
    // O cadastro recarregado oferece apenas selSerie. O fluxo publico ainda
    // permanece bloqueado; a fronteira HTTPS futura deve recusar ANTES do upload.
    let amb = a::ambiente();
    let mut respostas = a::abertura(arvore());
    respostas.push(a::ok(lista()));
    respostas.push(a::ok(cadastro("1", false)));
    respostas.push(a::ok(cadastro("1", true).replace(
        "infra_hash=FICTICIO",
        &format!("infra_hash={}", "b".repeat(64)),
    )));
    let mut upload_tentado = false;
    let (resultado, pedidos) = a::rodar_com(&amb, respostas, |sessao, nup, indice, rede| {
        let pdf = PdfLocal {
            nome: "prova.pdf".into(),
            tamanho_bytes: 68,
            sha256: "ficticio".into(),
        };
        super::super::fluxo_confirmar::executar_confirmacao_via_prevalidada(
            sessao,
            nup,
            None,
            "Anexo",
            &pdf,
            indice,
            rede,
            |_, _, _, _, _| {
                upload_tentado = true;
                panic!("formulario incompleto nao pode iniciar upload");
            },
        )
    });
    assert!(matches!(
        resultado,
        Err(
            super::super::fluxo_confirmar::ErroConfirmacaoExterna::CampoCadastroNaoOferecido(
                "txtDataElaboracao"
            )
        )
    ));
    assert!(!upload_tentado);
    assert_eq!(pedidos.len(), a::pedidos_da_abertura().len() + 3);
}

#[test]
fn link_externo_da_unb_com_id_procedimento_repetido_e_aceito_sem_reconstruir() {
    // Forma da cassette documento-incluir-unb (SEI 4.0.9, interação 13):
    // o link Externo de documento_escolher_tipo repete id_procedimento.
    // Ensaio live de 09/11/2026 recusava este link (link_externo_nao_conferido).
    let instalacao = Instalacao::interpretar(a::ORIGEM).unwrap();
    let href = format!(
        "controlador.php?acao=documento_receber&acao_origem=documento_escolher_tipo&acao_retorno=documento_escolher_tipo&id_procedimento=10&id_serie=-1&arvore=1&id_procedimento=10&infra_sistema=100000100&infra_unidade_atual=110000846&infra_hash={}",
        "f".repeat(64)
    );
    assert_eq!(
        conferir_destino(&instalacao, &href, "10").ok(),
        Some(format!("{}/{href}", a::ORIGEM))
    );
    for recusado in [
        href.replacen(
            "&arvore=1&id_procedimento=10&",
            "&arvore=1&id_procedimento=11&",
            1,
        ),
        href.replacen("&arvore=1&", "&id_procedimento=10&arvore=1&", 1),
    ] {
        assert!(
            matches!(
                conferir_destino(&instalacao, &recusado, "10"),
                Err(ErroPreviaExterna::LinkNaoConferido)
            ),
            "{recusado}"
        );
    }
}
