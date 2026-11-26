//! Busca sintética de número SEI: rota de pesquisa rápida e estado SQLite.
//! Somente loopback; sem consultas externas ou credenciais verdadeiras.
use crate::{Estado, Resposta, despachar, eh_controlador_sei, parametro_unico};
use iesei::documento_versoes::{ids_da_pesquisa_por_numero, ids_por_numero, ler as ler_documento};
use iesei::documentos_simulados::ids_por_nup;
use iesei::instalacao::Instalacao;
use rusqlite::Connection;

mod paginas;

/// Consulta o numero solicitado pela CLI. Nao cria documento nem
/// escolhe um ID arbitrario se a pesquisa encontra mais de um.
#[must_use]
pub fn responder(
    metodo: &str,
    alvo: &str,
    corpo: &str,
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
) -> Resposta {
    if !eh_controlador_sei(alvo)
        || parametro_unico(alvo, "acao") != Some("protocolo_pesquisa_rapida")
    {
        return despachar(metodo, alvo);
    }
    if metodo != "POST" {
        return Resposta::nova(Estado::MetodoNaoPermitido);
    }
    let Some(numero) = super::documentos::campo_formulario_unico(corpo, "txtPesquisaRapida")
        .filter(|numero| !numero.is_empty())
    else {
        return Resposta::nova(Estado::RequisicaoInvalida);
    };
    // A pesquisa rápida da CLI recebe NUP pontuado ou número SEI.
    // Primeiro os processos registrados; documentos históricos continuam
    // sendo consultáveis pelo número decimal sem colisão de escopos.
    let (ids, chave) = match ids_por_nup(banco, instalacao, cenario, &numero) {
        Ok(ids) if !ids.is_empty() => (ids, "id_protocolo"),
        Ok(_) if numero.bytes().all(|b| b.is_ascii_digit()) => {
            match ids_da_pesquisa_por_numero(banco, instalacao, cenario, &numero) {
                Ok(ids) => (ids, "id_documento"),
                Err(_) => return Resposta::nova(Estado::DadosInsuficientes),
            }
        }
        Ok(_) => (Vec::new(), "id_protocolo"),
        Err(_) => return Resposta::nova(Estado::DadosInsuficientes),
    };
    match ids.as_slice() {
        // SQLite parcial não prova a inexistência do NUP/documento.
        [] => Resposta::nova(Estado::DadosInsuficientes),
        [id] => Resposta::redirecionar(format!(
            "controlador.php?acao=procedimento_trabalhar&acao_origem=protocolo_pesquisa_rapida&{chave}={id}&infra_sistema=100000100&infra_unidade_atual=110000001&infra_hash={}",
            "c".repeat(64)
        )),
        _ => Resposta::nova(Estado::RequisicaoInvalida),
    }
}

/// Página HTML da mesma pesquisa servida pela interface Web observada,
/// reconstruída do estado (não de uma cópia de HTML de cassette).
fn formulario_de_pesquisa(unidade: &str, termo: &str, conteudo: &str) -> String {
    let hash = "c".repeat(64);
    format!(
        r#"<html><head><script>
        function navegar(inicio) {{
          document.getElementById('hdnInicio').value = inicio;
          document.getElementById('frmPesquisaProtocolo').submit();
        }}
        </script></head><body><a id="lnkInfraUnidade">{}</a>
        <form id="frmPesquisaProtocolo" name="frmPesquisaProtocolo" method="post"
        onsubmit="return onSubmitForm();"
        action="controlador.php?acao=protocolo_pesquisar&amp;acao_origem=protocolo_pesquisar&amp;infra_sistema=100000100&amp;infra_unidade_atual=110000001&amp;infra_hash={hash}#ancoraBarraPesquisa">
        <input type="hidden" name="hdnInfraTipoPagina" id="hdnInfraTipoPagina" value="1"/>
        <button type="submit" name="sbmPesquisar" id="sbmPesquisar" value="Pesquisar">Pesquisar</button>
        <input type="radio" id="optProcessos" name="rdoPesquisarEm" value="P"/>
        <input type="radio" id="optDocumentos" name="rdoPesquisarEm" value="D" checked="checked"/>
        <input type="checkbox" id="chkSinDocumentosGerados" name="chkSinDocumentosGerados" value="S" checked="checked"/>
        <input type="checkbox" id="chkSinDocumentosRecebidos" name="chkSinDocumentosRecebidos" value="S" checked="checked"/>
        <select multiple id="selOrgaoPesquisa" name="selOrgaoPesquisa[]">
        <option value="0" selected="selected">ORGAO</option></select>
        <input type="hidden" id="hdnIdUnidade" name="hdnIdUnidade" value=""/>
        <input type="hidden" id="hdnIdAssunto" name="hdnIdAssunto" value=""/>
        <input type="hidden" id="hdnIdAssinante" name="hdnIdAssinante" value=""/>
        <input type="hidden" id="hdnIdContato" name="hdnIdContato" value=""/>
        <input type="text" id="txtProtocoloPesquisa" name="txtProtocoloPesquisa" maxlength="50" value="{}"/>
        <input type="text" id="q" name="q" value=""/>
        <input type="hidden" id="hdnIdUsuarioGerador1" name="hdnIdUsuarioGerador1" value=""/>
        <input type="hidden" id="hdnIdUsuarioGerador2" name="hdnIdUsuarioGerador2" value=""/>
        <input type="hidden" id="hdnIdUsuarioGerador3" name="hdnIdUsuarioGerador3" value=""/>
        <div id="conteudo">{conteudo}</div>
        <input type="hidden" id="hdnInicio" name="hdnInicio" value="0"/>
        </form></body></html>"#,
        crate::escapar(unidade),
        crate::escapar(termo)
    )
}

/// O número SEI exato recupera a mesma identidade já presente nas versões.
/// Identidades ausentes ou ambíguas não são convertidas em "sem resultado".
fn documento_exato(
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
    termo: &str,
) -> Resposta {
    if !termo.bytes().all(|b| b.is_ascii_digit()) {
        return Resposta::nova(Estado::DadosInsuficientes);
    }
    let Ok(ids) = ids_por_numero(banco, instalacao, cenario, termo) else {
        return Resposta::nova(Estado::DadosInsuficientes);
    };
    let [id] = ids.as_slice() else {
        return Resposta::nova(Estado::DadosInsuficientes);
    };
    let Ok(Some(documento)) = ler_documento(banco, instalacao, cenario, id) else {
        return Resposta::nova(Estado::DadosInsuficientes);
    };
    Resposta::redirecionar(format!(
        "controlador.php?acao=procedimento_trabalhar&acao_origem=protocolo_pesquisar&id_procedimento={}&id_documento={id}&infra_sistema=100000100&infra_unidade_atual=110000001&infra_hash={}",
        documento.id_procedimento,
        crate::protocolo_sintetico::INFRA_HASH
    ))
}

// Ausência do controle equivale à primeira página; controle inválido não.
fn deslocamento_da_pesquisa(corpo: &str) -> Option<u64> {
    match corpo
        .split('&')
        .filter(|par| par.split('=').next() == Some("hdnInicio"))
        .count()
    {
        0 => Some(0),
        1 => super::documentos::campo_formulario_unico(corpo, "hdnInicio")?
            .parse()
            .ok(),
        _ => None,
    }
}

/// Pesquisa por protocolo na forma documento. Estado desconhecido retorna
/// 503, nunca `sem_resultado` por omissão de registro no SQLite.
#[must_use]
pub fn responder_pesquisa(
    metodo: &str,
    alvo: &str,
    corpo: &str,
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
) -> Resposta {
    if !eh_controlador_sei(alvo) || parametro_unico(alvo, "acao") != Some("protocolo_pesquisar") {
        return despachar(metodo, alvo);
    }
    let Ok(Some(controle)) = iesei::banco_cassettes::ler(banco, instalacao, cenario) else {
        return Resposta::nova(Estado::DadosInsuficientes);
    };
    match metodo {
        "GET" => Resposta::html(formulario_de_pesquisa(controle.unidade.as_str(), "", "")),
        "POST" => {
            // A chave legada não contém q: nunca responder uma consulta textual
            // com a observação de um protocolo. Só q ausente/vazio é equivalente.
            let mut textos = corpo
                .split('&')
                .filter(|par| par.split('=').next() == Some("q"));
            let texto = textos.next();
            if textos.next().is_some() {
                return Resposta::nova(Estado::RequisicaoInvalida);
            }
            if texto.is_some_and(|par| par != "q=") {
                return Resposta::nova(Estado::DadosInsuficientes);
            }
            let termo = super::documentos::campo_formulario_unico(corpo, "txtProtocoloPesquisa");
            let radio = super::documentos::campo_formulario_unico(corpo, "rdoPesquisarEm");
            let gerados =
                super::documentos::campo_formulario_unico(corpo, "chkSinDocumentosGerados");
            let recebidos =
                super::documentos::campo_formulario_unico(corpo, "chkSinDocumentosRecebidos");
            let Some(termo) = termo.filter(|t| !t.trim().is_empty()) else {
                return Resposta::nova(Estado::RequisicaoInvalida);
            };
            if radio.as_deref() != Some("D")
                || (gerados.as_deref() != Some("S") && recebidos.as_deref() != Some("S"))
            {
                return Resposta::nova(Estado::RequisicaoInvalida);
            }
            // O POST de navegar() repete o formulário, mudando só hdnInicio.
            // Sem a página observada, não inferimos uma faixa pelo total.
            let Some(deslocamento) = deslocamento_da_pesquisa(corpo) else {
                return Resposta::nova(Estado::RequisicaoInvalida);
            };
            let Ok(negativo) = iesei::pesquisa_documento::consultar(
                banco,
                instalacao,
                cenario,
                &controle.unidade,
                &termo,
            ) else {
                return Resposta::nova(Estado::DadosInsuficientes);
            };
            let pagina = iesei::pesquisa_documento::consultar_por_inicio(
                banco,
                instalacao,
                cenario,
                &controle.unidade,
                &termo,
                deslocamento,
            );
            match pagina {
                Ok(Some(pagina)) => {
                    let tamanho = if pagina.pagina == 1 {
                        pagina.fim
                    } else {
                        let Ok(Some(primeira)) = iesei::pesquisa_documento::consultar_pagina(
                            banco,
                            instalacao,
                            cenario,
                            &controle.unidade,
                            &termo,
                            1,
                        ) else {
                            return Resposta::nova(Estado::DadosInsuficientes);
                        };
                        if primeira.total_declarado != pagina.total_declarado {
                            return Resposta::nova(Estado::DadosInsuficientes);
                        }
                        primeira.fim
                    };
                    match paginas::renderizar(banco, instalacao, cenario, &pagina, tamanho) {
                        Ok(html) => Resposta::html(formulario_de_pesquisa(
                            controle.unidade.as_str(),
                            &termo,
                            &html,
                        )),
                        Err(estado) => Resposta::nova(estado),
                    }
                }
                Err(_) => Resposta::nova(Estado::DadosInsuficientes),
                Ok(None) if deslocamento != 0 => Resposta::nova(Estado::DadosInsuficientes),
                Ok(None) => match negativo {
                    Some(iesei::pesquisa_documento::Resultado::SemResultado) => {
                        Resposta::html(formulario_de_pesquisa(
                            controle.unidade.as_str(),
                            &termo,
                            "<div class='pesquisaSemResultado'>Nenhum resultado encontrado.</div>",
                        ))
                    }
                    None => documento_exato(banco, instalacao, cenario, &termo),
                },
            }
        }
        _ => Resposta::nova(Estado::MetodoNaoPermitido),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iesei::banco_cassettes::{self, ControleResumido, Linha};
    use iesei::observacao::{TabelaObservada, UnidadeExibida};

    #[test]
    fn deslocamento_ausente_nao_equivale_a_controle_invalido() {
        for (corpo, esperado) in [
            ("q=", Some(0)),
            ("hdnInicio=0", Some(0)),
            ("hdnInicio=10", Some(10)),
            ("hdnInicio=%31%30", Some(10)),
            ("hdnInicio=", None),
            ("hdnInicio", None),
            ("hdnInicio=%ZZ", None),
            ("hdnInicio=%FF", None),
            ("hdnInicio=-1", None),
            ("hdnInicio=18446744073709551616", None),
            ("hdnInicio=10&hdnInicio=0", None),
            ("hdnInicio&hdnInicio=10", None),
        ] {
            assert_eq!(deslocamento_da_pesquisa(corpo), esperado, "{corpo}");
        }
    }

    #[test]
    fn registro_negativo_na_unidade_nao_inventa_resultado_de_outro_termo() {
        let mut banco = Connection::open_in_memory().unwrap();
        banco_cassettes::iniciar(&banco).unwrap();
        let instalacao = Instalacao::interpretar("https://sei.test/sei").unwrap();
        let unidade = UnidadeExibida::nova("TESTE").unwrap();
        banco_cassettes::guardar(
            &mut banco,
            "cenario",
            &ControleResumido {
                instalacao: instalacao.clone(),
                unidade: unidade.clone(),
                recebidos: TabelaObservada::<Linha>::nova(Some(0), None, vec![]).unwrap(),
                gerados: TabelaObservada::<Linha>::nova(Some(0), None, vec![]).unwrap(),
            },
        )
        .unwrap();
        iesei::pesquisa_documento::registrar_sem_resultado(
            &banco,
            &instalacao,
            "cenario",
            &unidade,
            "DOC-NAO-ACHADO",
        )
        .unwrap();
        let url = "/sei/controlador.php?acao=protocolo_pesquisar";
        let tela = responder_pesquisa("GET", url, "", &banco, &instalacao, "cenario");
        assert_eq!(tela.estado, Estado::Ok);
        assert!(tela.corpo.contains("frmPesquisaProtocolo"));
        assert!(tela.corpo.contains("optDocumentos"));
        assert!(tela.corpo.contains("name=\"q\""));
        let buscar = |termo: &str| {
            responder_pesquisa(
                "POST",
                url,
                &format!("rdoPesquisarEm=D&chkSinDocumentosGerados=S&txtProtocoloPesquisa={termo}"),
                &banco,
                &instalacao,
                "cenario",
            )
        };
        let observado = buscar("DOC-NAO-ACHADO");
        assert_eq!(observado.estado, Estado::Ok);
        assert!(observado.corpo.contains("pesquisaSemResultado"));
        assert_eq!(buscar("OUTRO").estado, Estado::DadosInsuficientes);
        // A observação negativa de protocolo não prova a mesma consulta com q.
        for (criterios, esperado) in [
            ("txtProtocoloPesquisa=DOC-NAO-ACHADO&q=", Estado::Ok),
            (
                "txtProtocoloPesquisa=DOC-NAO-ACHADO&hdnInicio=10&hdnInicio=0",
                Estado::RequisicaoInvalida,
            ),
            (
                "txtProtocoloPesquisa=DOC-NAO-ACHADO&hdnInicio=%ZZ",
                Estado::RequisicaoInvalida,
            ),
            (
                "txtProtocoloPesquisa=DOC-NAO-ACHADO&q=contrato",
                Estado::DadosInsuficientes,
            ),
            (
                "txtProtocoloPesquisa=&q=contrato",
                Estado::DadosInsuficientes,
            ),
            (
                "txtProtocoloPesquisa=DOC-NAO-ACHADO&q=licita%E7%E3o",
                Estado::DadosInsuficientes,
            ),
            (
                "txtProtocoloPesquisa=DOC-NAO-ACHADO&q=&q=contrato",
                Estado::RequisicaoInvalida,
            ),
        ] {
            let resposta = responder_pesquisa(
                "POST",
                url,
                &format!("rdoPesquisarEm=D&chkSinDocumentosGerados=S&{criterios}"),
                &banco,
                &instalacao,
                "cenario",
            );
            assert_eq!(resposta.estado, esperado, "{criterios}");
        }
        // A segunda página contradiz o zero explícito, ainda que não haja
        // primeira página positiva. Nenhum deslocamento pode renderizá-la.
        iesei::pesquisa_documento::registrar_pagina(
            &mut banco,
            &instalacao,
            "cenario",
            &unidade,
            "DOC-NAO-ACHADO",
            &iesei::pesquisa_documento::PaginaObservada {
                pagina: 2,
                inicio: 11,
                fim: 20,
                total_declarado: 20,
                linhas: vec![],
            },
        )
        .unwrap();
        for deslocamento in [0, 10] {
            let resposta = responder_pesquisa(
                "POST",
                url,
                &format!(
                    "rdoPesquisarEm=D&chkSinDocumentosGerados=S&txtProtocoloPesquisa=DOC-NAO-ACHADO&hdnInicio={deslocamento}"
                ),
                &banco,
                &instalacao,
                "cenario",
            );
            assert_eq!(resposta.estado, Estado::DadosInsuficientes);
            assert!(!resposta.corpo.contains("pesquisaSemResultado"));
        }
        assert_eq!(
            responder_pesquisa(
                "POST",
                url,
                "rdoPesquisarEm=P&txtProtocoloPesquisa=DOC-NAO-ACHADO",
                &banco,
                &instalacao,
                "cenario",
            )
            .estado,
            Estado::RequisicaoInvalida
        );
    }
}
