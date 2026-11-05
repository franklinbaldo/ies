---
type: InterfaceObservation
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
estiver presente e habilitado. Restrito exige hipótese selecionável
na lista que o cadastro oferece (que pode depender de AJAX);
**não presumir** que uma hipótese existe. Interessados fornecidos
devem ser resolvidos univocamente pela busca da própria tela.

A amostra registra `btnSalvar` como botão de ação no navegador;
o Todos observa que não integra o payload: o formulário é submetido
pelo script. A escrita marca `hdnFlagProcedimentoCadastro=2`,
transmite a descrição, os campos da tela e os hidden de listas.
Na implementação IES não executar JS: derivar os valores e a ação
a partir dos atributos e da estrutura HTML conhecida.

## Resposta, identidade e risco

Na escrita, o oráculo observa redirecionamento para a árvore;
o `Location` traz `id_procedimento`. O `302` isolado **não**
certifica o resultado. A leitura seguinte da árvore deve fornecer
NUP não vazio e `id_procedimento` igual ao redirecionado.
Para restrito ou interessados, conferir também o cadastro
gravado quando a tela o permitir. Não inventar NUP com base no ID.

Depois do POST criador, falha de transporte, redirect inesperado,
ausência de identificador e leitura inconclusiva exigem
`efeito_incerto` (saída 3), **sem retry**. A abertura do cadastro
e a prévia não podem enviar o POST criador.

## Lacunas

Faltam captura e replay IES por versão, orçamento preciso medido
por cenário, validação de campos obrigatórios em cada instalação
e prévia live autenticada. Nenhuma escrita em treinamento está
autorizada nesta issue até o texto literal “escrita autorizada”.
