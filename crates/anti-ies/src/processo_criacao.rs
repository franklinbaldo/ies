//! Criacao de processos em SQLite efemero: lista, cadastro, POST unico e
//! projecoes de processo/propriedades, sem acessar SEI externo.
//! A forma das telas e hipotese revisavel; o ID/NUP tem fonte unica no banco.
use crate::{Estado, Resposta, controlar, escapar, parametro_unico, protocolo_sintetico};
use iesei::criacao_processo::{self, Criado, TipoDeProcesso};
use iesei::instalacao::Instalacao;
use rusqlite::Connection;
use std::fmt::Write as _;

const HASH: &str = protocolo_sintetico::INFRA_HASH;

/// Um cenario sem tipos observados nao anuncia criacao no menu.
pub(crate) fn acrescentar_menu(
    resposta: Resposta,
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
) -> Resposta {
    if resposta.estado != Estado::Ok {
        return resposta;
    }
    match criacao_processo::listar(banco, instalacao, cenario) {
        Ok(Some(_)) => {
            let mut resposta = resposta;
            if resposta.corpo.contains("</body>") {
                let link = format!(
                    "<a href='controlador.php?acao=procedimento_escolher_tipo&amp;infra_hash={HASH}'>Iniciar Processo</a>"
                );
                resposta.corpo = resposta
                    .corpo
                    .replacen("</body>", &format!("{link}</body>"), 1);
            }
            resposta
        }
        Ok(None) => resposta,
        Err(_) => Resposta::nova(Estado::DadosInsuficientes),
    }
}

fn tela_de_tipos(tipos: &[TipoDeProcesso]) -> Resposta {
    let mut html = format!(
        "<html><body><form id='frmProcedimentoEscolherTipo' method='post' \
         action='controlador.php?acao=procedimento_escolher_tipo&amp;infra_hash={HASH}'>\
         <input type='hidden' name='hdnIdTipoProcedimento' value=''>\
         <input type='hidden' name='hdnFiltroTipoProcedimento' value='U'>\
         </form><button onclick=\"exibirTiposProcedimento('U')\">Todos</button>\
         <table id='tblTipoProcedimento'><tbody>"
    );
    for tipo in tipos {
        write!(
            html,
            "<tr><td><a class='ancoraOpcao' href='controlador.php?acao=procedimento_gerar&amp;id_tipo_procedimento={}&amp;infra_hash={HASH}'>{}</a></td></tr>",
            escapar(&tipo.id),
            escapar(&tipo.nome)
        )
        .expect("formatação String");
    }
    html.push_str("</tbody></table></body></html>");
    Resposta::html(html)
}

fn cadastro(tipo: &TipoDeProcesso) -> Resposta {
    Resposta::html(format!(
        "<html><body><form id='frmProcedimentoCadastro' method='post' \
         action='controlador.php?acao=procedimento_gerar&amp;infra_hash={HASH}'>\
         <input type='hidden' name='hdnIdTipoProcedimento' value='{}'>\
         <input type='hidden' name='hdnFlagProcedimentoCadastro' value='0'>\
         <input type='hidden' name='hdnAssuntos' value=''>\
         <input type='text' name='txtDescricao' value=''>\
         <input type='radio' name='rdoNivelAcesso' value='0' checked>\
         <input type='radio' name='rdoNivelAcesso' value='1'>\
         <select name='selAssuntos'><option value='{}'>{}</option></select>\
         <select name='selHipoteseLegal'><option value='31'>Base Legal Teste</option></select>\
         </form></body></html>",
        escapar(&tipo.id),
        escapar(&tipo.id_assunto),
        escapar(&tipo.assunto),
    ))
}

fn campo_unico<'a>(campos: &'a [(String, String)], nome: &str) -> Option<&'a str> {
    let mut valores = campos.iter().filter(|(chave, _)| chave == nome);
    let (_, valor) = valores.next()?;
    valores.next().is_none().then_some(valor.as_str())
}

/// Inspeciona exclusivamente o corpo form-urlencoded; rejeita duplicidade,
/// codificacao malformada e campos faltantes antes de tocar no banco.
fn campos_do_formulario(corpo: &str) -> Option<Vec<(String, String)>> {
    if corpo.len() > 16_384 {
        return None;
    }
    corpo
        .split('&')
        .map(|parte| {
            let (nome, valor) = parte.split_once('=')?;
            Some((
                crate::processo_envio::decodificar_formulario(nome)?,
                crate::processo_envio::decodificar_formulario(valor)?,
            ))
        })
        .collect()
}

fn confirmar(corpo: &str, banco: &Connection, instalacao: &Instalacao, cenario: &str) -> Resposta {
    let Some(campos) = campos_do_formulario(corpo) else {
        return Resposta::nova(Estado::RequisicaoInvalida);
    };
    let (Some(id_tipo), Some(especificacao), Some("2"), Some("0"), Some(assunto)) = (
        campo_unico(&campos, "hdnIdTipoProcedimento"),
        campo_unico(&campos, "txtDescricao"),
        campo_unico(&campos, "hdnFlagProcedimentoCadastro"),
        campo_unico(&campos, "rdoNivelAcesso"),
        campo_unico(&campos, "hdnAssuntos"),
    ) else {
        return Resposta::nova(Estado::RequisicaoInvalida);
    };
    if campo_unico(&campos, "hdnInteressadosProcedimento").is_some_and(|valor| !valor.is_empty()) {
        // Resolucao de interessados nao faz parte desta fatia.
        return Resposta::nova(Estado::RequisicaoInvalida);
    }
    let tipo = match criacao_processo::consultar(banco, instalacao, cenario, id_tipo) {
        Ok(Some(tipo)) => tipo,
        Ok(None) => return Resposta::nova(Estado::RequisicaoInvalida),
        Err(_) => return Resposta::nova(Estado::DadosInsuficientes),
    };
    if assunto != format!("{}±{}", tipo.id_assunto, tipo.assunto) {
        return Resposta::nova(Estado::RequisicaoInvalida);
    }
    let unidade = match iesei::banco_cassettes::ler(banco, instalacao, cenario) {
        Ok(Some(controle)) => controle.unidade.as_str().to_owned(),
        Ok(None) | Err(_) => return Resposta::nova(Estado::DadosInsuficientes),
    };
    match criacao_processo::criar(banco, instalacao, cenario, id_tipo, especificacao, &unidade) {
        Ok(criado) => Resposta::redirecionar(format!(
            "controlador.php?acao=procedimento_trabalhar&id_procedimento={}",
            criado.id_procedimento
        )),
        Err(criacao_processo::Falha::DadoInvalido) => Resposta::nova(Estado::RequisicaoInvalida),
        Err(criacao_processo::Falha::Sqlite(_)) => Resposta::nova(Estado::DadosInsuficientes),
    }
}

/// Projeta propriedades somente de um processo realmente criado no SQLite.
/// A leitura do IES confere NUP, tipo, descricao e nivel nesta mesma identidade.
fn propriedades_html(criado: &Criado) -> Resposta {
    let p = escapar(&criado.id_procedimento);
    let nup = escapar(&criado.nup);
    let tipo_id = escapar(&criado.tipo.id);
    let tipo = escapar(&criado.tipo.nome);
    let assunto_id = escapar(&criado.tipo.id_assunto);
    let assunto = escapar(&criado.tipo.assunto);
    let descricao = escapar(&criado.especificacao);
    Resposta::html(format!(
        "<html><body><form id='frmProcedimentoCadastro'>\
         <input id='hdnProtocoloFormatado' value='{nup}'>\
         <input id='hdnIdProcedimento' value='{p}'>\
         <input id='hdnIdTipoProcedimento' value='{tipo_id}'>\
         <select id='selTipoProcedimento'><option value='{tipo_id}' selected>{tipo}</option></select>\
         <input id='txtDescricao' value='{descricao}'>\
         <input id='hdnAssuntos' value=''>\
         <input id='hdnInteressadosProcedimento' value=''>\
         <select id='selAssuntos'><option value='{assunto_id}' selected>{assunto}</option></select>\
         <select id='selInteressadosProcedimento'></select>\
         <textarea id='txaObservacoes'></textarea>\
         <input type='radio' name='rdoNivelAcesso' id='optPublico' value='0' checked>\
         <select id='selHipoteseLegal'><option value='null' selected>Selecione</option></select>\
         </form></body></html>"
    ))
}

pub(crate) fn propriedades(
    metodo: &str,
    alvo: &str,
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
) -> Resposta {
    if metodo != "GET" {
        return Resposta::nova(Estado::MetodoNaoPermitido);
    }
    let Some(id) = parametro_unico(alvo, "id_procedimento") else {
        return Resposta::nova(Estado::RequisicaoInvalida);
    };
    match criacao_processo::ler_criado(banco, instalacao, cenario, id) {
        Ok(Some(criado)) => propriedades_html(&criado),
        Ok(None) => Resposta::nova(Estado::NaoEncontrado),
        Err(_) => Resposta::nova(Estado::DadosInsuficientes),
    }
}

/// Lista por GET, e permite uma unica escrita de cadastro pela CLI.
pub(crate) fn responder(
    metodo: &str,
    alvo: &str,
    corpo: &str,
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
) -> Resposta {
    match (metodo, parametro_unico(alvo, "acao")) {
        ("GET", Some("procedimento_escolher_tipo")) => {
            match criacao_processo::listar(banco, instalacao, cenario) {
                Ok(Some(tipos)) => tela_de_tipos(&tipos),
                Ok(None) | Err(_) => Resposta::nova(Estado::DadosInsuficientes),
            }
        }
        ("GET", Some("procedimento_gerar")) => {
            let Some(id) = parametro_unico(alvo, "id_tipo_procedimento") else {
                return Resposta::nova(Estado::RequisicaoInvalida);
            };
            match criacao_processo::consultar(banco, instalacao, cenario, id) {
                Ok(Some(tipo)) => cadastro(&tipo),
                Ok(None) => Resposta::nova(Estado::NaoEncontrado),
                Err(_) => Resposta::nova(Estado::DadosInsuficientes),
            }
        }
        ("POST", Some("procedimento_gerar")) => confirmar(corpo, banco, instalacao, cenario),
        ("GET", _) | ("POST", _) => Resposta::nova(Estado::NaoImplementado),
        _ => Resposta::nova(Estado::MetodoNaoPermitido),
    }
}

/// Rota do Controle com menu derivado da tabela de tipos.
pub(crate) fn controle_com_menu(
    metodo: &str,
    alvo: &str,
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
) -> Resposta {
    acrescentar_menu(
        controlar(metodo, alvo, banco, instalacao, cenario),
        banco,
        instalacao,
        cenario,
    )
}
