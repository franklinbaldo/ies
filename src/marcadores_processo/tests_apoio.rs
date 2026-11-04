//! Fixtures SINTÉTICAS com a estrutura observada na `UnB` 4.0.9
//! (04/11/2026) e em GO (cassette `marcador-escritas-go`): o link
//! `andamento_marcador_gerenciar` da barra do nó `PROCESSO` e a tela
//! "Marcadores do Processo" com `table#tblMarcadores`, os botões de escrita
//! e o `form#frmGerenciarMarcador`. Hosts, hashes, ids, nomes, textos,
//! usuários e datas inventados; nenhum valor servido.
use std::fmt::Write as _;

pub(crate) const HASH_MARCADORES: &str =
    "6666666666666666666666666666666666666666666666666666666666666666";
const HASH_OUTRO: &str = "4444444444444444444444444444444444444444444444444444444444444444";
const CONTEXTO: &str = "infra_sistema=100000100&infra_unidade_atual=110000001";

/// O link relativo da tela, como a barra o publica (`&` cru).
pub(crate) fn link_dos_marcadores(id: &str) -> String {
    format!(
        "controlador.php?acao=andamento_marcador_gerenciar&acao_origem=procedimento_visualizar&acao_retorno=arvore_visualizar&id_procedimento={id}&arvore=1&{CONTEXTO}&infra_hash={HASH_MARCADORES}"
    )
}

/// Um marcador da tela: id, nome, cor, texto (vazio: célula vazia),
/// sigla do usuário (vazia: célula vazia) e data/hora.
pub(crate) struct Linha<'a> {
    pub(crate) id: &'a str,
    pub(crate) nome: &'a str,
    pub(crate) cor: &'a str,
    pub(crate) texto: &'a str,
    pub(crate) usuario: &'a str,
    pub(crate) data_hora: &'a str,
}

/// Os dois marcadores do processo padrão (o caso do proprietário: um
/// processo com mais de um marcador).
pub(crate) fn dois() -> [Linha<'static>; 2] {
    [
        Linha {
            id: "501",
            nome: "Revis\u{e3}o",
            cor: "azul",
            texto: "",
            usuario: "pessoa1",
            data_hora: "03/11/2026 10:15",
        },
        Linha {
            id: "502",
            nome: "Pendente",
            cor: "amarelo",
            texto: "Texto sint\u{e9}tico &amp; anota\u{e7}\u{e3}o",
            usuario: "pessoa2",
            data_hora: "02/11/2026 09:00",
        },
    ]
}

/// Uma linha `tr.infraTrClara` na forma servida (`k` é a posição).
pub(crate) fn linha(k: usize, l: &Linha<'_>) -> String {
    let Linha {
        id,
        nome,
        cor,
        texto,
        usuario,
        data_hora,
    } = l;
    let usuario = if usuario.is_empty() {
        String::new()
    } else {
        format!(
            "<a alt=\"Pessoa Sintetica\" title=\"Pessoa Sintetica\" class=\"ancoraSigla\">{usuario}</a>"
        )
    };
    format!(
        "<tr class=\"infraTrClara\"><td valign=\"center\"><a id=\"lnkInfraID-{id}\" name=\"ID-{id}\"></a>\
         <input  class=\"infraCheckbox\" id=\"chkInfraItem{k}\" name=\"chkInfraItem{k}\" tabindex=\"1000\" title=\"{nome}\" type=\"checkbox\" value=\"{id}\" onclick=\"infraSelecionarItens(this,'Infra');\"/></td>\
         <td align=\"left\" valign=\"center\"><a href=\"#\" tabindex=\"1000\"><img src=\"svg/marcador_{cor}.svg?11\" title=\"{nome}\" alt=\"{nome}\" class=\"infraImg\" style=\"vertical-align: middle;\"/></a>&nbsp;{nome}</td>\n\
         <td id=\"tdAndamentoMarcador{id}\" valign=\"center\">{texto}</td>\n\
         <td align=\"center\"  valign=\"center\">{usuario}</td><td align=\"center\" valign=\"center\">{data_hora}</td>\n\
         <td align=\"center\" valign=\"center\"><a href=\"controlador.php?acao=andamento_marcador_alterar&acao_origem=andamento_marcador_gerenciar&acao_retorno=andamento_marcador_gerenciar&id_marcador={id}&id_procedimento=10&{CONTEXTO}&infra_hash={HASH_OUTRO}\" >\
         <img src=\"svg/marcador_anotacao.svg?11\" title=\"Alterar Texto do Marcador no Processo\" alt=\"Alterar\" class=\"infraImg\" /></a>&nbsp;\
         <a href=\"#ID-{id}\" onclick=\"infraLimparFormatarTrAcessada(this.parentNode.parentNode);acaoRemover('{id}','{nome}');\" >\
         <img src=\"/infra_css/svg/remover.svg\" title=\"Remover Marcador do Processo\" alt=\"Remover\" class=\"infraImg\" /></a>&nbsp;</td></tr>\n"
    )
}

fn ordenacao(rotulo: &str) -> String {
    format!(
        "<th class=\"infraTh\" width=\"10%\">\n<div class=\"infraDivOrdenacao\">\n<div class=\"infraDivRotuloOrdenacao\">{rotulo}</div>\n\
         <div class=\"infraDivSetaOrdenacao\"><a href=\"javascript:void(0);\" onclick=\"infraAcaoOrdenar('X','ASC','Infra',null);\" tabindex=\"1000\">\
         <img src=\"/infra_css/svg/seta_acima.svg\" title=\"Ordenar\" alt=\"Ordenar\" class=\"infraImgOrdenacao\" /></a></div>\n</div>\n</th>\n"
    )
}

/// O cabeçalho observado: seleção, Marcador, Texto, Usuário, Data/Hora, Ações.
pub(crate) fn cabecalho() -> String {
    format!(
        "<tr><th class=\"infraTh\" width=\"1%\"><label id=\"lblInfraCheck\" for=\"lnkInfraCheck\" accesskey=\";\"></label>\n\
         <a href=\"javascript:void(0);\" id=\"lnkInfraCheck\" onclick=\"infraSelecaoMultipla('Infra');\" tabindex=\"1000\">\
         <img src=\"/infra_css/svg/check.svg\" id=\"imgInfraCheck\" title=\"Selecionar Tudo\" alt=\"Selecionar Tudo\" class=\"infraImg\" /></a></th>\n\
         {}<th class=\"infraTh\">Texto</th>{}{}<th class=\"infraTh\" width=\"10%\">A\u{e7}\u{f5}es</th></tr>\n",
        ordenacao("Marcador"),
        ordenacao("Usu\u{e1}rio"),
        ordenacao("Data/Hora")
    )
}

/// A tabela com `legenda` e as linhas; `None` tira a tabela da tela.
pub(crate) fn tabela(legenda: &str, linhas: &[Linha<'_>]) -> String {
    let mut corpo = String::new();
    for (k, l) in linhas.iter().enumerate() {
        corpo.push_str(&linha(k, l));
    }
    format!(
        "<table id=\"tblMarcadores\" width=\"99%\" class=\"infraTable\" summary=\"Marcadores\">\n\
         <caption class=\"infraCaption\">{legenda}</caption>{}{corpo}</table>",
        cabecalho()
    )
}

/// A legenda observada: `Marcadores (N registro|registros):`.
pub(crate) fn legenda(n: usize) -> String {
    let plural = if n == 1 { "registro" } else { "registros" };
    format!("Marcadores ({n} {plural}):")
}

/// A tela inteira com `tabela` (HTML) na área da tabela.
pub(crate) fn tela_com(nup: &str, id: &str, tabela: &str) -> String {
    let mut html = format!(
        "<!DOCTYPE html><html lang=\"pt-br\"><head><meta http-equiv=\"Content-Type\" content=\"text/html; charset=iso-8859-1\" />\n\
         <title>SEI - Marcadores do Processo {nup}</title>\n\
         <script type=\"text/javascript\">\nfunction acaoRemocaoMultipla(){{\n  if (document.getElementById('hdnInfraItensSelecionados').value==''){{\n    alert('Nenhum Marcador selecionado.');\n    return;\n  }}\n\
           document.getElementById('frmGerenciarMarcador').action='controlador.php?acao=andamento_marcador_remover&acao_origem=andamento_marcador_gerenciar&id_procedimento={id}&{CONTEXTO}&infra_hash={HASH_OUTRO}';\n\
           document.getElementById('frmGerenciarMarcador').submit();\n}}\n</script></head><body>\n"
    );
    let _ = write!(
        html,
        "<form id=\"frmGerenciarMarcador\" method=\"post\" onsubmit=\"return OnSubmitForm();\" action=\"controlador.php?acao=andamento_marcador_gerenciar&acao_origem=andamento_marcador_gerenciar&id_procedimento={id}&{CONTEXTO}&infra_hash={HASH_OUTRO}\">\n\
         <input type=\"hidden\" id=\"hdnInfraTipoPagina\" name=\"hdnInfraTipoPagina\" value=\"1\" />\n\
         <div id=\"divInfraBarraComandosSuperior\" class=\"infraBarraComandos\">\
         <button tabindex=\"1000\" type=\"button\" id=\"btnAdicionar\" value=\"Adicionar\" onclick=\"location.href='controlador.php?acao=andamento_marcador_cadastrar&acao_origem=andamento_marcador_gerenciar&id_procedimento={id}&{CONTEXTO}&infra_hash={HASH_OUTRO}'\" class=\"infraButton\">Adicionar</button>\
         <button tabindex=\"1000\" type=\"button\" id=\"btnRemover\" value=\"Remover\" onclick=\"acaoRemocaoMultipla();\" class=\"infraButton\">Remover</button></div>\n\
         <div id=\"divInfraAreaPaginacaoSuperior\" class=\"infraAreaPaginacao\">\n</div>\n\
         <div id=\"divInfraAreaTabela\" class=\"infraAreaTabela\" >\n{tabela}\n</div>\n\
         <input type=\"hidden\" id=\"hdnInfraNroItens\" name=\"hdnInfraNroItens\" value=\"2\" />\n\
         <input type=\"hidden\" id=\"hdnInfraItensSelecionados\" name=\"hdnInfraItensSelecionados\" value=\"\" />\n\
         </form>\n</body></html>"
    );
    html
}

/// A tela do processo `nup`/`id` com os dois marcadores.
pub(crate) fn tela_dos_marcadores(nup: &str, id: &str) -> String {
    let linhas = dois();
    tela_com(nup, id, &tabela(&legenda(linhas.len()), &linhas))
}
