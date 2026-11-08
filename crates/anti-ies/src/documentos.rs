//! Rotas de navegação e criação de documento no anti-IES (#246).
//! Modelo local sintético; HTML vem de SQLite, jamais de uma cassette congelada.
use super::{Estado, Resposta, eh_controlador_sei, escapar, parametro_unico};
use iesei::documentos_simulados::{self as modelo, Falha};
use iesei::instalacao::Instalacao;
use rusqlite::Connection;
use std::fmt::Write as _;

fn id_da_rota<'a>(alvo: &'a str, nome: &str) -> Option<&'a str> {
    parametro_unico(alvo, nome).filter(|v| v.bytes().all(|b| b.is_ascii_digit()))
}

pub(crate) fn campo_formulario_unico(corpo: &str, nome: &str) -> Option<String> {
    let mut campos = corpo.split('&').filter_map(|campo| campo.split_once('='));
    let valor = campos.find(|(n, _)| *n == nome)?.1;
    let existe_repetido = corpo
        .split('&')
        .filter_map(|parte| parte.split_once('='))
        .filter(|(n, _)| *n == nome)
        .count()
        != 1;
    if existe_repetido {
        return None;
    }
    let mut bytes = Vec::new();
    let mut iter = valor.as_bytes().iter().copied();
    while let Some(byte) = iter.next() {
        match byte {
            b'+' => bytes.push(b' '),
            b'%' => {
                let a = iter.next()?;
                let b = iter.next()?;
                let digito = |c: u8| (c as char).to_digit(16);
                bytes.push(u8::try_from(digito(a)? * 16 + digito(b)?).ok()?);
            }
            c if c.is_ascii() => bytes.push(c),
            _ => return None,
        }
    }
    String::from_utf8(bytes).ok()
}

fn erros_de_modelo(falha: &Falha) -> Resposta {
    match falha {
        Falha::ProcessoIndisponivel => Resposta::nova(Estado::NaoEncontrado),
        Falha::TipoIndisponivel | Falha::EntradaInvalida => {
            Resposta::nova(Estado::RequisicaoInvalida)
        }
        Falha::Sqlite(_) => Resposta::nova(Estado::DadosInsuficientes),
    }
}

fn pagina_do_processo(
    banco: &Connection,
    origem: &Instalacao,
    cenario: &str,
    id: &str,
) -> Resposta {
    let Ok(Some(processo)) = modelo::ler_processo(banco, origem, cenario, id) else {
        return Resposta::nova(Estado::NaoEncontrado);
    };
    let Ok(documentos) = modelo::documentos(banco, origem, cenario, id) else {
        return Resposta::nova(Estado::DadosInsuficientes);
    };
    let mut html = format!(
        "<html><body><a id='lnkInfraUnidade' href='#'>{}</a>\
         <iframe id='ifrArvore' name='ifrArvore' src='controlador.php?acao=procedimento_visualizar&amp;id_procedimento={id}&amp;infra_hash={}'></iframe>\
         <span id='numeroProcesso'>{}</span><span id='unidade'>{}</span>\
         <a id='incluirDocumento' href='controlador.php?acao=documento_escolher_tipo&amp;id_procedimento={}'>Incluir Documento</a>\
         <ul id='documentos'>",
        escapar(&processo.unidade),
        "e".repeat(64),
        escapar(&processo.nup),
        escapar(&processo.unidade),
        id,
    );
    for doc in documentos {
        write!(
            html,
            "<li data-id='{}' data-numero-sei='{}'>{} {}</li>",
            doc.id,
            doc.numero_sei,
            escapar(&doc.tipo),
            doc.numero_sei
        )
        .expect("formatação String");
    }
    html.push_str("</ul></body></html>");
    Resposta::html(html)
}

/// A árvore de processo não deve tomar a rota de versões de documento.
pub(crate) fn eh_arvore_do_processo(alvo: &str) -> bool {
    eh_controlador_sei(alvo)
        && parametro_unico(alvo, "id_procedimento").is_some()
        && parametro_unico(alvo, "id_documento").is_none()
        && parametro_unico(alvo, "id_protocolo").is_none()
}

/// Rotas da criação: outras formas de `procedimento_visualizar`
/// continuam pertencendo ao leitor de versões, sem colisão por nome.
pub(crate) fn eh_rota_documentos(alvo: &str) -> bool {
    match parametro_unico(alvo, "acao") {
        Some("documento_escolher_tipo" | "documento_gerar" | "arvore_visualizar") => {
            eh_controlador_sei(alvo)
        }
        Some("procedimento_visualizar") => eh_arvore_do_processo(alvo),
        _ => false,
    }
}

/// A árvore é projetada dos documentos persistidos, não de HTML congelado.
/// A mesma leitura é usada antes e depois do POST criador da #246.
fn pagina_da_arvore(banco: &Connection, origem: &Instalacao, cenario: &str, id: &str) -> Resposta {
    let Ok(Some(processo)) = modelo::ler_processo(banco, origem, cenario, id) else {
        return Resposta::nova(Estado::NaoEncontrado);
    };
    let Ok(documentos) = modelo::documentos(banco, origem, cenario, id) else {
        return Resposta::nova(Estado::DadosInsuficientes);
    };
    let mut html = String::from("<html><head><script>\nvar Nos = new Array();\n");
    let nup = escapar_js(&processo.nup);
    writeln!(
        html,
        "Nos[0] = new infraArvoreNo(\"PROCESSO\",\"{id}\",null,\
         \"controlador.php?acao=arvore_visualizar&id_procedimento={id}\",\
         \"ifrVisualizacao\",\"{nup}\",\"Processo simulado\",\
         \"svg/processo.svg\",\"svg/processo.svg\",\"svg/processo.svg\",\
         true,true,\"noVisitado\",null,\"noVisitado\",\"{nup}\");"
    )
    .expect("formatação String");
    let link = format!(
        "<a href=\"controlador.php?acao=documento_escolher_tipo&amp;id_procedimento={id}&amp;infra_hash={}\">Incluir Documento</a>",
        "e".repeat(64)
    );
    let unidade_html = escapar(&processo.unidade);
    let situacao = format!(
        "Processo aberto somente na unidade <a alt=\"{unidade_html}\" title=\"{unidade_html}\" class=\"ancoraSigla\">{unidade_html}</a>.<br />"
    );
    writeln!(html, "Nos[0].acoes = '{}';", escapar_js(&link)).expect("formatação String");
    html.push_str("Nos[0].src = '';\n");
    writeln!(html, "Nos[0].html = '{}';", escapar_js(&situacao)).expect("formatação String");
    for (indice, doc) in documentos.iter().enumerate() {
        let indice = indice + 1;
        let rotulo = escapar_js(&format!("{} {}", doc.tipo, doc.numero_sei));
        let numero = escapar_js(&doc.numero_sei);
        writeln!(
            html,
            "Nos[{indice}] = new infraArvoreNo(\"DOCUMENTO\",\"{}\",\"{id}\",\
             \"about:blank\",\"ifrVisualizacao\",\"{rotulo}\",\"Documento interno\",\
             \"svg/documento_interno.svg\",\"svg/documento_interno.svg\",\
             \"svg/documento_interno.svg\",true,false,null,null,\"noVisitado\",\"{numero}\");",
            doc.id
        )
        .expect("formatação String");
        writeln!(html, "Nos[{indice}].acoes = '';").expect("formatação String");
        writeln!(html, "Nos[{indice}].src = '';").expect("formatação String");
        writeln!(html, "Nos[{indice}].html = '';").expect("formatação String");
    }
    html.push_str("</script></head><body><div id=\"divArvore\"></div></body></html>");
    Resposta::html(html)
}

/// Conteúdo do literal JS publicado no simulador, sem executar o script.
fn escapar_js(texto: &str) -> String {
    texto
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\'', "\\'")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}

fn pagina_da_lista(
    banco: &Connection,
    origem: &Instalacao,
    cenario: &str,
    id: &str,
    mostrar_todos: bool,
) -> Resposta {
    let Ok(Some(_)) = modelo::ler_processo(banco, origem, cenario, id) else {
        return Resposta::nova(Estado::NaoEncontrado);
    };
    let Ok(tipos) = modelo::tipos(banco, origem, cenario, id) else {
        return Resposta::nova(Estado::DadosInsuficientes);
    };
    // A primeira tela simula a lista recente; o POST "Exibir todos" devolve
    // a lista completa sem criar documento nem modificar o SQLite.
    let tem_filtro = tipos.len() > 1;
    let hash = "e".repeat(64); // Somente navegação sintética do simulador.
    let mut html = format!(
        "<html><head><script>function escolher(id) {{\
         document.getElementById('hdnIdSerie').value=id;\
         document.getElementById('frmDocumentoEscolherTipo').submit();\
         }}function exibirSeries(tipo) {{\
         document.getElementById('hdnFiltroSerie').value=tipo;\
         document.getElementById('frmDocumentoEscolherTipo').submit();\
         }}</script></head><body>\
         <form id='frmDocumentoEscolherTipo' method='post' \
         action='controlador.php?acao=documento_escolher_tipo&amp;id_procedimento={id}&amp;infra_hash={hash}'>\
         <input type='hidden' id='hdnIdSerie' name='hdnIdSerie' value=''>"
    );
    if tem_filtro {
        html.push_str("<input type='hidden' id='hdnFiltroSerie' name='hdnFiltroSerie' value='U'>");
    }
    html.push_str("</form><table id='tblSeries'>");
    for tipo in tipos
        .iter()
        .take(if mostrar_todos { tipos.len() } else { 1 })
    {
        write!(
            html,
            "<tr><td><a href='#' onclick='escolher({})'>{}</a></td></tr>",
            tipo.id,
            escapar(&tipo.nome)
        )
        .expect("formatação String");
    }
    html.push_str("</table>");
    if tem_filtro && !mostrar_todos {
        html.push_str(
            "<a id='ancExibirSeries' href='#' onclick=\"exibirSeries('T')\">Exibir todos</a>",
        );
    }
    html.push_str("</body></html>");
    Resposta::html(html)
}

fn cadastro(
    banco: &Connection,
    origem: &Instalacao,
    cenario: &str,
    id: &str,
    id_serie: &str,
) -> Resposta {
    let Ok(Some(_)) = modelo::ler_processo(banco, origem, cenario, id) else {
        return Resposta::nova(Estado::NaoEncontrado);
    };
    let Ok(tipos) = modelo::tipos(banco, origem, cenario, id) else {
        return Resposta::nova(Estado::DadosInsuficientes);
    };
    let Some(tipo) = tipos.iter().find(|t| t.id == id_serie) else {
        return Resposta::nova(Estado::NaoEncontrado);
    };
    let hash = "e".repeat(64); // Somente para a navegação no simulador.
    Resposta::html(format!(
        "<html><head><script>function confirmarDados() {{ submeter(); }}\
         function submeter() {{ document.getElementById('hdnFlagDocumentoCadastro').value='2';\
         document.getElementById('frmDocumentoCadastro').submit(); }}</script></head><body>\
         <label id='lblSerieTitulo'>{}</label>\
         <form id='frmDocumentoCadastro' method='post' \
         action='controlador.php?acao=documento_gerar&amp;id_procedimento={id}&amp;id_serie={id_serie}&amp;infra_hash={hash}'>\
         <input type='hidden' name='hdnIdSerie' value='{id_serie}'>\
         <input type='hidden' name='hdnFlagDocumentoCadastro' value='1'>\
         <input type='radio' id='optPublico' name='rdoNivelAcesso' value='0'>\
         <input type='text' id='txtDescricao' name='txtDescricao'>\
         </form><button type='button' id='btnSalvar' name='btnSalvar' onclick='confirmarDados()'>Salvar</button></body></html>",
        escapar(&tipo.nome),
    ))
}

/// GET navega; POST altera apenas o estado SQLite da simulação.
/// Tipos, identidade e documentos pertencem ao mesmo cenário/instalação.
pub(crate) fn atender(
    metodo: &str,
    alvo: &str,
    corpo: &str,
    banco: &Connection,
    origem: &Instalacao,
    cenario: &str,
) -> Resposta {
    if !eh_controlador_sei(alvo) {
        return Resposta::nova(Estado::NaoEncontrado);
    }
    let Some(acao) = parametro_unico(alvo, "acao") else {
        return Resposta::nova(Estado::RequisicaoInvalida);
    };
    if !matches!(
        acao,
        "documento_gerar"
            | "documento_escolher_tipo"
            | "procedimento_trabalhar"
            | "procedimento_visualizar"
            | "arvore_visualizar"
    ) {
        return Resposta::nova(Estado::NaoImplementado);
    }
    let id_procedimento = id_da_rota(alvo, "id_procedimento");
    let id_protocolo = id_da_rota(alvo, "id_protocolo");
    let id = match (id_procedimento, id_protocolo) {
        (Some(id), None) => id,
        (None, Some(id)) if acao == "procedimento_trabalhar" => id,
        _ => return Resposta::nova(Estado::RequisicaoInvalida),
    };
    match (acao, metodo) {
        ("procedimento_trabalhar" | "arvore_visualizar", "GET") => {
            pagina_do_processo(banco, origem, cenario, id)
        }
        ("procedimento_visualizar", "GET") => pagina_da_arvore(banco, origem, cenario, id),
        ("documento_escolher_tipo", "GET") => pagina_da_lista(banco, origem, cenario, id, false),
        ("documento_escolher_tipo", "POST") => {
            // Um filtro T pede a lista inteira; a escolha de série continua
            // sendo uma operação de navegação diferente da criação.
            let filtro_presente = corpo
                .split('&')
                .filter_map(|campo| campo.split_once('='))
                .any(|(nome, _)| nome == "hdnFiltroSerie");
            let filtro = campo_formulario_unico(corpo, "hdnFiltroSerie");
            if filtro_presente && filtro.is_none() {
                return Resposta::nova(Estado::RequisicaoInvalida);
            }
            match filtro.as_deref() {
                Some("T") => {
                    if campo_formulario_unico(corpo, "hdnIdSerie")
                        .is_some_and(|serie| !serie.is_empty())
                    {
                        return Resposta::nova(Estado::RequisicaoInvalida);
                    }
                    return pagina_da_lista(banco, origem, cenario, id, true);
                }
                None | Some("U") => {}
                _ => return Resposta::nova(Estado::RequisicaoInvalida),
            }
            let Some(serie) = campo_formulario_unico(corpo, "hdnIdSerie") else {
                return Resposta::nova(Estado::RequisicaoInvalida);
            };
            if serie.is_empty() || !serie.bytes().all(|b| b.is_ascii_digit()) {
                return Resposta::nova(Estado::RequisicaoInvalida);
            }
            // A mesma conferencia do cadastro valida processo e serie.
            let resposta = cadastro(banco, origem, cenario, id, &serie);
            if resposta.estado != Estado::Ok {
                return resposta;
            }
            Resposta::redirecionar(format!(
                "controlador.php?acao=documento_gerar&id_procedimento={id}&id_serie={serie}&infra_hash={}",
                "e".repeat(64)
            ))
        }
        ("documento_gerar", "GET") => {
            let Some(serie) = id_da_rota(alvo, "id_serie") else {
                return Resposta::nova(Estado::RequisicaoInvalida);
            };
            cadastro(banco, origem, cenario, id, serie)
        }
        ("documento_gerar", "POST") => {
            let Some(serie) = id_da_rota(alvo, "id_serie") else {
                return Resposta::nova(Estado::RequisicaoInvalida);
            };
            if campo_formulario_unico(corpo, "hdnIdSerie").as_deref() != Some(serie)
                || campo_formulario_unico(corpo, "hdnFlagDocumentoCadastro").as_deref() != Some("2")
                || campo_formulario_unico(corpo, "rdoNivelAcesso").as_deref() != Some("0")
            {
                return Resposta::nova(Estado::RequisicaoInvalida);
            }
            let Some(descricao) = campo_formulario_unico(corpo, "txtDescricao") else {
                return Resposta::nova(Estado::RequisicaoInvalida);
            };
            match modelo::criar(banco, origem, cenario, id, serie, &descricao) {
                Ok(novo) => Resposta::redirecionar(format!(
                    "controlador.php?acao=arvore_visualizar&id_procedimento={id}&id_documento={}",
                    novo.id,
                )),
                Err(falha) => erros_de_modelo(&falha),
            }
        }
        _ => Resposta::nova(Estado::MetodoNaoPermitido),
    }
}
