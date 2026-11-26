//! Ciência documental sintética: tela de nó, GET mutador único e lista relida.
//! O estado é o mesmo SQLite descartável do IES; nenhum cookie ou link real.
use crate::{Estado, Resposta, eh_controlador_sei, parametro_unico};
use iesei::ciencia_documental::{self as modelo, Estado as Ciencia, Resultado};
use iesei::instalacao::Instalacao;
use rusqlite::Connection;

/// Ação oferecida na barra do documento, nunca no processo vizinho.
pub(crate) fn barra(ciencia: &Ciencia) -> &'static str {
    match ciencia {
        Ciencia::Pendente => {
            r##"<a href="#" onclick="cienciaDocumento();" title="Ciência">Ciência</a>"##
        }
        Ciencia::Registrada { .. } => "",
    }
}

/// Uma lista só é renderizada depois de uma atividade realmente persistida.
pub(crate) fn acao_da_lista(
    indice: usize,
    documento: &str,
    processo: &str,
    ciencia: &Ciencia,
) -> String {
    let Ciencia::Registrada { .. } = ciencia else {
        return String::new();
    };
    let hash = crate::protocolo_sintetico::INFRA_HASH;
    format!(
        r#"NosAcoes[{indice}] = new infraArvoreAcao("CIENCIAS","CD{documento}","{documento}","controlador.php?acao=protocolo_ciencia_listar&id_procedimento={processo}&id_documento={documento}&infra_hash={hash}","ifrVisualizacao","Visualizar Ciências","svg/ciencia.svg",true);
"#
    )
}

pub(crate) fn estado(
    banco: &Connection,
    origem: &Instalacao,
    cenario: &str,
    documento: &str,
    processo: &str,
) -> Result<Option<Ciencia>, Estado> {
    modelo::consultar(banco, origem, cenario, documento, processo)
        .map_err(|_| Estado::DadosInsuficientes)
}

/// Tela de nó, separada da árvore. Não reproduz HTML gravado de cassettes.
pub(crate) fn tela(
    metodo: &str,
    alvo: &str,
    banco: &Connection,
    origem: &Instalacao,
    cenario: &str,
) -> Option<Resposta> {
    if !eh_controlador_sei(alvo) || parametro_unico(alvo, "acao") != Some("arvore_visualizar") {
        return None;
    }
    let (Some(documento), Some(processo)) = (
        parametro_unico(alvo, "id_documento"),
        parametro_unico(alvo, "id_procedimento"),
    ) else {
        return None;
    };
    if !documento.bytes().all(|b| b.is_ascii_digit())
        || !processo.bytes().all(|b| b.is_ascii_digit())
    {
        return Some(Resposta::nova(Estado::RequisicaoInvalida));
    }
    let existente = match estado(banco, origem, cenario, documento, processo) {
        Ok(Some(ciencia)) => ciencia,
        Ok(None) => return None,
        Err(erro) => return Some(Resposta::nova(erro)),
    };
    if metodo != "GET" {
        return Some(Resposta::nova(Estado::MetodoNaoPermitido));
    }
    let hash = crate::protocolo_sintetico::INFRA_HASH;
    let link = if matches!(existente, Ciencia::Pendente) {
        format!(
            "var linkCienciaDocumento = 'controlador.php?acao=documento_ciencia&id_documento={documento}&infra_hash={hash}';"
        )
    } else {
        String::new()
    };
    // A tela do nó é compartilhada: oferecer edição não apaga a ciência.
    let editor = match links_do_editor(banco, origem, cenario, documento, processo) {
        Ok(editor) => editor,
        Err(estado) => return Some(Resposta::nova(estado)),
    };
    Some(Resposta::html(format!(
        "<html><head><title>SEI - Visualizar Árvore</title><script type=\"text/javascript\">\n{link}\n{editor}\n</script></head><body></body></html>"
    )))
}


// Hipótese sintética da tela observada: ambas as URLs são publicadas na
// mesma identidade. A consulta AJAX continua independente da oferta.
fn links_do_editor(
    banco: &Connection,
    origem: &Instalacao,
    cenario: &str,
    documento: &str,
    processo: &str,
) -> Result<String, Estado> {
    use iesei::cancelamento_documento::Situacao;
    use iesei::editor_documento::consultar_oferta;

    let oferta = consultar_oferta(banco, origem, cenario, documento)
        .map_err(|_| Estado::DadosInsuficientes)?;
    if !oferta.is_some_and(|o| o.editavel)
        || !matches!(
            crate::documento_cancelar::estado(banco, origem, cenario, documento, processo)?,
            Some(Situacao::Disponivel)
        )
    {
        return Ok(String::new());
    }
    let hash = crate::protocolo_sintetico::INFRA_HASH;
    Ok(format!(
        "objAjaxVerificacaoAssinatura = new infraAjaxComplementar(null,'controlador_ajax.php?acao_ajax=documento_verificar_assinatura&id_documento={documento}&infra_hash={hash}');\nvar linkEditarConteudo = 'controlador.php?acao=editor_montar&id_procedimento={processo}&id_documento={documento}&infra_hash={hash}';"
    ))
}

/// Rotas cuja consulta à lista não modifica o documento.
/// O GET `documento_ciencia` é uma única escrita, nunca um "GET" inocente.
pub(crate) fn responder(
    metodo: &str,
    alvo: &str,
    banco: &Connection,
    origem: &Instalacao,
    cenario: &str,
) -> Resposta {
    if !eh_controlador_sei(alvo) {
        return Resposta::nova(Estado::NaoEncontrado);
    }
    if metodo != "GET" {
        return Resposta::nova(Estado::MetodoNaoPermitido);
    }
    let Some(documento) =
        parametro_unico(alvo, "id_documento").filter(|id| id.bytes().all(|b| b.is_ascii_digit()))
    else {
        return Resposta::nova(Estado::RequisicaoInvalida);
    };
    match parametro_unico(alvo, "acao") {
        Some("documento_ciencia") => {
            let processo = match modelo::processo_do_documento(banco, origem, cenario, documento) {
                Ok(Some(processo)) => processo,
                Ok(None) => return Resposta::nova(Estado::NaoEncontrado),
                Err(_) => return Resposta::nova(Estado::DadosInsuficientes),
            };
            if parametro_unico(alvo, "id_procedimento").is_some_and(|id| id != processo) {
                return Resposta::nova(Estado::RequisicaoInvalida);
            }
            match modelo::registrar(banco, origem, cenario, documento, &processo) {
                Ok(Resultado::Registrada { id_atividade }) => Resposta::html(format!(
                    "<html><head><script>function inicializar(){{ atualizarArvore(         'controlador.php?acao=procedimento_visualizar&id_procedimento={processo}&id_documento={documento}&documento_visualizar_ciencias=1&id_atividade={id_atividade}#ID-{id_atividade}'); }}</script></head></html>"
                )),
                Ok(Resultado::JaRegistrada) => Resposta::html(
                    "<html><body><textarea id='txaInfraValidacao' style='display:none'>\
                     Usuário já deu ciência neste documento.</textarea></body></html>"
                        .to_owned(),
                ),
                Ok(Resultado::DocumentoNaoEncontrado) => Resposta::nova(Estado::NaoEncontrado),
                Err(_) => Resposta::nova(Estado::DadosInsuficientes),
            }
        }
        Some("protocolo_ciencia_listar") => {
            let Some(processo) = parametro_unico(alvo, "id_procedimento")
                .filter(|id| id.bytes().all(|b| b.is_ascii_digit()))
            else {
                return Resposta::nova(Estado::RequisicaoInvalida);
            };
            match estado(banco, origem, cenario, documento, processo) {
                Ok(Some(Ciencia::Registrada { id_atividade })) => {
                    // A escrita foi persistida, mas a projeção da Lista pode
                    // ainda não publicar a atividade. Trata-se de hipótese
                    // sintética de efeito incerto, nunca de perda de gravação.
                    match modelo::lista_invisivel(banco, origem, cenario, documento) {
                        Ok(true) => {
                            return Resposta::html(
                                "<html><table id='tblAndamentos'><caption>Lista de Ciências:</caption><tr><th>Atividade</th><th>Usuário</th></tr></table></html>"
                                    .to_owned(),
                            );
                        }
                        Ok(false) => {}
                        Err(_) => return Resposta::nova(Estado::DadosInsuficientes),
                    }
                    let Ok(usuario) =
                        modelo::usuario_da_ciencia(banco, origem, cenario, documento, processo)
                    else {
                        return Resposta::nova(Estado::DadosInsuficientes);
                    };
                    // A lista só nomeia o autor quando a sessão sintética
                    // o identificou. Revisores do documento não são autores
                    // inferidos da ciência.
                    let autoria = usuario.as_deref().map_or_else(
                        || "<td></td>".to_owned(),
                        |nome| format!("<td>{}</td>", crate::escapar(nome)),
                    );
                    Resposta::html(format!(
                        "<html><table id='tblAndamentos'><caption>Lista de Ciências:</caption><tr><th>Atividade</th><th>Usuário</th></tr><tr><td><input type='checkbox' name='chkInfraItem' value='{id_atividade}' /></td>{autoria}</tr></table></html>"
                    ))
                }
                Ok(Some(Ciencia::Pendente)) => Resposta::nova(Estado::DadosInsuficientes),
                Ok(None) => Resposta::nova(Estado::NaoEncontrado),
                Err(erro) => Resposta::nova(erro),
            }
        }
        _ => Resposta::nova(Estado::NaoEncontrado),
    }
}

#[cfg(test)]
mod testes_editor {
    use super::*;
    use iesei::documento_versoes::{DocumentoSimulado, Revisao, guardar, ler};
    use iesei::editor_documento::{Oferta, registrar_oferta};
    use std::num::NonZeroU32;

    #[test]
    fn tela_compoe_ciencia_e_editor_sem_inventar_oferta() {
        let origem = Instalacao::interpretar("https://sei.test/sei").unwrap();
        let mut banco = Connection::open_in_memory().unwrap();
        iesei::banco_cassettes::iniciar(&banco).unwrap();
        let documento = DocumentoSimulado {
            instalacao: origem.clone(),
            id_documento: "21".into(),
            id_procedimento: "10".into(),
            numero_processo: "00001.000001/2026-01".into(),
            numero: "0000101".into(),
            tipo: "Despacho".into(),
            revisoes: vec![Revisao {
                versao: NonZeroU32::new(1).unwrap(),
                usuario: "Teste".into(),
                unidade: "TESTE".into(),
                modificacao: "26/11/2026".into(),
                html_corpo: "<p>Antes</p>".into(),
            }],
        };
        guardar(&mut banco, "ensaio", &documento).unwrap();
        let alvo = "/sei/controlador.php?acao=arvore_visualizar&id_procedimento=10&id_documento=21";
        let pedir = |alvo, cenario| {
            crate::rotas::responder("GET", alvo, b"", &banco, &origem, cenario)
        };
        let sem_oferta = pedir(alvo, "ensaio");
        assert_eq!(sem_oferta.estado, Estado::Ok);
        assert!(sem_oferta.corpo.contains("linkCienciaDocumento"));
        assert!(!sem_oferta.corpo.contains("linkEditarConteudo"));
        for (editavel, assinaturas) in [(false, 0), (true, 0), (true, 2)] {
            registrar_oferta(
                &banco,
                &origem,
                "ensaio",
                "21",
                Oferta {
                    editavel,
                    assinaturas,
                },
            )
            .unwrap();
            let resposta = pedir(alvo, "ensaio");
            assert_eq!(resposta.estado, Estado::Ok);
            assert!(resposta.corpo.contains("linkCienciaDocumento"));
            assert_eq!(resposta.corpo.contains("linkEditarConteudo"), editavel);
            assert_eq!(
                resposta.corpo.contains(
                    "acao_ajax=documento_verificar_assinatura&id_documento=21&infra_hash="
                ),
                editavel
            );
            assert_eq!(
                resposta.corpo.contains(
                    "acao=editor_montar&id_procedimento=10&id_documento=21&infra_hash="
                ),
                editavel
            );
        }
        let outro = alvo.replace("id_procedimento=10", "id_procedimento=11");
        assert!(!pedir(&outro, "ensaio").corpo.contains("linkEditarConteudo"));
        assert!(!pedir(alvo, "outro").corpo.contains("linkEditarConteudo"));
        iesei::cancelamento_documento::aplicar(&banco, &origem, "ensaio", "21", "10", "Teste")
            .unwrap();
        assert!(!pedir(alvo, "ensaio").corpo.contains("linkEditarConteudo"));
        assert_eq!(
            ler(&banco, &origem, "ensaio", "21").unwrap(),
            Some(documento)
        );
    }
}
