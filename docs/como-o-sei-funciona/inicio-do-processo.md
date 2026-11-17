---
type: SEIObservation
title: Iniciar processo pela Web do SEI
operation: processo.criar
source: todos-src-e-capturas-publicas
status: observacao-indireta
---

# Início de processo — observação estrutural (issue #245)

Esta observação é **indireta**, extraída do oráculo Todos em
`src/todos/processos.py` (funções `_tipos_da_lista`,
`_tipo_na_lista`, `_cadastro_conferido`, `_criar_direto` e `criar`)
e da captura `tests/paginas_reais/procedimento_gerar.html` (SEI 4.0.9).
Não houve sessão ao vivo do IES nesta etapa. Os cassettes do IES
`processo-criar-listas-unb`, `processo-criar-previa-unb` e
`processo-criar-restrito-unb` remetem ao acervo do Todos; seus
resumos **não** equivalem a replay IES.

## Escolha do tipo

A entrada publicada pelo menu é `procedimento_escolher_tipo`.
A lista tem tabela `tblTipoProcedimento` e âncoras
`ancoraOpcao`. Há duas estruturas observadas:

- SEI 4.0: `href` para `procedimento_gerar` com
  `id_tipo_procedimento`; abrir o cadastro é um GET.
- SEI 4.1/5.0: `href="#"` e `onclick="escolher(ID)"`;
  o formulário `frmProcedimentoEscolherTipo` preenche
  `hdnIdTipoProcedimento` e é enviado para abrir o cadastro.
  **Esse POST é uma leitura de cadastro, não a escrita criadora.**

A lista inicialmente pode conter só os tipos mais usados. O controle
`exibirTiposProcedimento('T')` pede todos por
`hdnFiltroTipoProcedimento=T`; o controle com `'U'` indica que
a lista completa está à vista. Não declarar tipo ausente antes de
conferir a lista completa. Nome repetido, ação ambígua ou sem link
identificável são recusas. IDs são opacos; o nome deve casar com
um único tipo oferecido.

## Cadastro

O cadastro usa `frmProcedimentoCadastro`. A amostra pública contém:

- `hdnIdTipoProcedimento` (tipo escolhido);
- `txtDescricao` (especificação do usuário);
- `selAssuntos` e `hdnAssuntos` (assunto sugerido pelo tipo);
- `txtInteressadoProcedimento`, `selInteressadosProcedimento`
  e `hdnInteressadosProcedimento` (interessados resolvidos na tela);
- `txaObservacoes` (observações, fora do escopo inicial);
- `rdoNivelAcesso` (rádios de nível) e `selHipoteseLegal`
  (hipótese ao optar por restrito).

O Todos exige que o cadastro reconhecido tenha
`hdnFlagProcedimentoCadastro`, tipo idêntico ao escolhido e
`hdnAssuntos`. Sem assuntos fornecidos pelo usuário, o oráculo
aceita exatamente um assunto sugerido; caso contrário, recusa.
Um nível solicitado só pode ser enviado se o rádio correspondente
estiver presente e habilitado. Pela semântica HTML do formulário,
`select disabled` não participa de seleção: opção de assunto visível
num seletor desabilitado não é assunto oferecido; em nível restrito,
idem para a hipótese legal. A guarda é negativa e foi testada com
HTML sintético derivado da captura publicada, **sem alegar que essas
instalações já apresentaram selects desabilitados**. Restrito exige hipótese selecionável
na lista que o cadastro oferece (que pode depender de AJAX);
**não presumir** que uma hipótese existe. Interessados fornecidos
devem ser resolvidos univocamente pela busca da própria tela.
No retorno AJAX de autocomplete, a lista publicada é `<itens>`,
com os candidatos em elementos `<item>` internos. Itens que não
pertencem à raiz observada não oferecem identidade selecionável.
A regressão #245 injeta um `<item>` externo **sintético** para impedir
a aceitação de identidade alheia; não atribui essa estrutura a uma
instalação real.

A amostra registra `btnSalvar` como botão de ação no navegador;
o Todos observa que não integra o payload: o formulário é submetido
pelo script. A escrita marca `hdnFlagProcedimentoCadastro=2`,
transmite a descrição, os campos da tela e os hidden de listas.
Na implementação IES não executar JS: derivar os valores e a ação
a partir dos atributos e da estrutura HTML conhecida.

## Limite publicado da especificação — observação adicional 06/11/2026

A captura `tests/paginas_reais/procedimento_gerar.html` do Todos
(`323eb01`, SEI 4.0.9) contém o `input` `txtDescricao` com
`maxlength="100"` e a máscara servida `infraMascaraTexto(this,event,100)`.
A captura `procedimento_gerar_go_5_0_3.html` também publica
`maxlength="100"`, sem demonstrar regra universal para todas as
instalações. A [issue Todos #1248](https://github.com/franklinbaldo/todos/issues/1248)
relatou recusa explícita do SEI para especificação superior a 100
caracteres; é **relato externo**, não ensaio live do IES, e não se
acessou a instalação de produção mencionada.

A validação do IES deve utilizar somente o `maxlength` numérico servido
pelo formulário **da própria sessão**. Na ausência do atributo, não
inventar limite; se o atributo é ilegível, falhar fechado. Para a
restrição nativa de `input`, comprimento é contado em unidades UTF-16
(como `String.length` da interface Web), não em bytes UTF-8. A prévia
deve rejeitar um valor que exceda o limite observado sem enviar o POST
criador, e a confirmação deve passar pela mesma guarda.

## Resposta, identidade e risco

Na escrita, o oráculo observa redirecionamento para a árvore;
o `Location` traz `id_procedimento`. O `302` isolado **não**
certifica o resultado. A leitura seguinte da árvore deve fornecer
NUP não vazio e `id_procedimento` igual ao redirecionado.
Para restrito ou interessados, conferir também o cadastro
gravado quando a tela o permitir. Não inventar NUP com base no ID.

Depois do POST criador, falha de transporte, redirect inesperado,
ausência de identificador e leitura inconclusiva exigem
`nao_confirmado` (saída 1), **sem retry**. A abertura do cadastro
e a prévia não podem enviar o POST criador.

## Lacunas

A observação por versão foi complementada por testes de transporte sintético
e replay de 15 cassettes originais, com 83 respostas classificadas
([relatório](../validacao-replay/2026-11-05-processo-criar-acervo.md)).
A validação dos campos em cada instalação e a prévia live autenticada ainda faltam. Em 05/11/2026, o proprietário
autorizou escrita **somente em instalações de treinamento** na issue #245.
A autorização não comprova execução: a validação live segue NOT_RUN.
Antes de criar, é obrigatório observar a prévia, enviar um único
`--confirmar` e verificar o novo processo por leitura; documentar
qualquer resíduo no ambiente de treinamento.

## Hipótese legal no cadastro restrito — observação indireta 05/11/2026

A captura do Todos em `tests/paginas_reais/procedimento_gerar.html`
(SEI 4.0.9, commit `323eb01`) publica no JavaScript servido o
construtor literal `infraAjaxMontarSelect('selHipoteseLegal', URL)`,
com a ação AJAX `hipotese_legal_select_nome_base_legal`. O `select`
inicial pode conter apenas o placeholder `null`; isso **não**
prova ausência de hipóteses. O navegador solicita opções ao selecionar
o nível restrito. O oráculo `todos.documentos.hipoteses_do_cadastro`
faz a consulta diretamente, por POST de leitura, com os campos
`primeiroItemValor=null`, `primeiroItemDescricao= `,
`valorItemSelecionado=` e `staNivelAcesso=1`. A resposta
é XML com `<select><option value="...">...</option></select>`.
Não executar o JavaScript: reconhecer a declaração literal de URL,
derivar a consulta com `formulario_web` e interpretar a resposta.

A CLI admite `--hipotese-legal ID#NOME` somente para nível restrito:
o ID e o nome devem corresponder exatamente à opção habilitada oferecida
no XML observado da sessão. Sem escolha explícita, **somente uma**
opção efetiva é selecionável; zero ou várias opções geram recusa
antes do POST criador. Um placeholder `null` não é opção efetiva.
A lista é evidência indireta; o replay do acervo original foi executado,
mas o ensaio autenticado de prévia → confirmação única → leitura
permanece NOT_RUN. URLs assinadas e credenciais não são publicadas.

## Diagnóstico seguro quando há opções AJAX múltiplas

O XML de treinamento no acervo da UnB contém múltiplas hipóteses
habilitadas. A leitura do IES pode informar os pares `ID#NOME`
que o próprio XML ofereceu, sem seguir URLs adicionais. A falta de
escolha continua impedindo o POST criador: essa informação ajuda a
formular uma **nova prévia** explícita, não autoriza confirmação.
Somente opções habilitadas, com IDs distintos e texto sem controles,
podem aparecer na saída. O IES não publica URL AJAX assinada,
`infra_hash` nem cookie. Essa evidência de cassettes não substitui
um ensaio autenticado em treinamento, ainda NOT_RUN.
