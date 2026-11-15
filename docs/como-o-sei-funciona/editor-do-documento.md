---
type: SEIObservation
title: Editor de documentos do SEI Web
description: Observação estrutural do editor e limites da evidência antes da escrita
---

# Editor do documento: observação, não contrato de escrita

Esta nota atende à fase **observar** da [#89](https://github.com/franklinbaldo/ies/issues/89).
Não autoriza abrir o editor ou gravar: a abertura pode afetar assinaturas.
Atribuir uma funcionalidade ao Todos não a torna implementada no IES.
Nenhum JavaScript foi executado.

## Fontes e limites

- [Índice de evidências](../../contracts/evidencias/documento-salvar.md):
  sete cards `documento-escritas-*`, passo 7, para UnB, PEN1–PEN4,
  UFRJ e GO. Os cards registram três `editor_montar` e dois
  `editor_salvar` (UnB, UFRJ, PEN1, PEN2) ou
  `editor_salvar_conteudo` (GO, PEN3, PEN4) por cenário;
  o cenário inteiro tem entre 65 e 70 requisições, não esse número
  de salvamentos.
- [Catálogo do acervo](../cassettes.md): sete cards referenciam
  ZIPs imutáveis no Internet Archive sem todos os corpos de POST. Em
  **15/11/2026 foram conferidos os dois ZIPs UnB/GO e seus membros
  originais**, mas os cinco restantes continuam apenas catalogados.
- [Todos @323eb01](https://github.com/franklinbaldo/todos/tree/323eb01):
  `src/todos/documentos.py`, `src/todos/sei/editor5.py`,
  `tests/test_editor5_010.py`,
  `tests/test_editor5_documento_010.py`,
  `tests/test_resposta_do_editor_e_plano_010.py`,
  `tests/test_documento_versionado_1238.py` e
  `tests/paginas_reais/editor_montar.html`.
  Os testes usam fixtures e SEI falso; não validam escrita Rust ao vivo.

## Confronto estrutural direto: UnB e GO (15/11/2026)

Offline, sem GET a instalações ou qualquer escrita. A fonte é o membro
`documento-escritas-*.json» do ZIP original apontado por cada card.
Verifiquei SHA-1 do ZIP, SHA-256 normalizado da cassette, `schema_version: 1»,
`complete: true» e a origem:

| Origem | ZIP SHA-1 | JSON SHA-256 (LF) | GETs `editor_montar» |
| --- | --- | --- | --- |
| [UnB](../../cassettes/unb/documento-escritas-unb--36a904749e20.md) | `c7a4af8d9d70ddc3599cdd9bccbb98e6d726db78» | `36a904749e2042c769cb37034f2e21f05b1c6c02432ac458f6f8bad902d04c02» | 12, 20, 27 |
| [GO](../../cassettes/go/documento-escritas-go--16f83179ecaa.md) | `42a08cd3714bb863d136ed98e517221510a31982» | `16f83179ecaabecb1de24598a8aeda51088954c116f998514cfa8cc3a8045100» | 15, 22, 29 |

- **UnB (SEI 4):** três GETs HTTP 200, ISO-8859-1, com
  `form#frmEditor» `method=post», `hdnVersao» (1, 2, 2),
  `hdnIgnorarNovaVersao=N», `hdnSiglaUnidade»,
  `hdnInfraPrefixoCookie», `iframe#ifrEditorSalvar» e três seções
  `txaEditor_368», `txaEditor_370», `txaEditor_372».
  Não aparece `txaEditor_1» nem `output#assinaturasAtuais» do ensaio.
- **GO (SEI 5):** três GETs HTTP 200, ISO-8859-1, com
  `window.INFRA_EDITOR_CONFIG», `initialData», `rootsAttributes»,
  `somenteLeitura» e `editor_salvar_conteudo». Não há `form#frmEditor»,
  `iframe» ou `textarea» da família SEI 4.
- As capturas contêm duas respostas de POST `editor_salvar» em UnB e duas
  `editor_salvar_conteudo» em GO, mas não demonstram o payload integral,
  efeito, assinaturas removidas nem promoção de versão. `documento_alterar»
  é formulário de metadados, distinto do editor de conteúdo.

O código `documento.salvar --confirmar» opera somente em `sei.test»;
o parser foi estendido apenas para **reconhecer** as duas famílias e
**recusar** sua gravação sem protocolo validado. O teste ignorado
`replay_editor_nativo_no_acervo» confronta a classificação estrutural
com seis respostas originais e não executa a CLI completa ou o anti-IES.
Os confrontos de transição/escrita e o live continuam **NOT_RUN**.

**Risco para ensaio posterior:** GET de processo/editor pode registrar
recebimento ou remover assinaturas. Nenhuma abertura live foi autorizada;
orçamento de rede para esta fatia: **0 GET / 0 POST**.

## O que a barra da árvore publica

O [leitor da barra](../../src/arvore_processo/barra_do_documento.rs)
reconhece a declaração `onclick="editarConteudo('N');"` para
não assinado e `onclick="editarConteudo('S');"` para assinado.
A segunda forma sinaliza risco de perda da assinatura ao editar.
Se ausente ou não reconhecida, não se presume permissão de edição.
A barra **declara** a ação; não se deve executar o JavaScript,
inventar a URL de edição ou supor que a assinatura seja imutável.
A consulta `documento_versao_listar` é ação distinta da mesma barra.

## `editor_montar` no SEI 4 / CKEditor 4

**Origem do detalhe:** fixture do Todos com tela UnB 4.0.9
[`editor_montar.html`](https://github.com/franklinbaldo/todos/blob/323eb01/tests/paginas_reais/editor_montar.html)
e leitor `_ler_editor` em `documentos.py`.

- Formulário `form#frmEditor` oculto, `method="post"`, alvo
  `ifrEditorSalvar`, `action` servido da operação
  `editor/editor_processar.php` / `editor_salvar`.
- Uma `textarea` `txaEditor_*` por seção; campos ocultos
  `hdnVersao` e `hdnIgnorarNovaVersao=N`.
  `iframe#ifrEditorSalvar` recebe a resposta e possui
  `onload="verificarSalvamento();"`.
- Scripts publicados incluem `editor/ck/ckeditor.js`,
  `CKEDITOR.replace` por seção, opções de somente leitura,
  estilos e comandos de salvar. É obrigatório inferir gravabilidade
  **da estrutura e da configuração servidas**, não do número ou
  posição de `textarea`. A fixture tem três seções: não generalizar.
- O Todos exige correspondência do documento da tela/formulário,
  versão disponível e seção principal gravável; recusa tela de
  documento diferente, corpo ilegível ou ausência de seção editável.
  As demais seções são preservadas no envio.

`documento_alterar` é a tela de **cadastro/metadados**,
diferente de `editor_montar` (conteúdo) e `editor_salvar`
(POST de conteúdo). A fixture `documento_alterar.html` do Todos
não comprova a operação de salvar conteúdo.

## `editor_montar` no SEI 5 / CKEditor 5

**Origem do detalhe:** `src/todos/sei/editor5.py` e
`tests/test_editor5_010.py` de Todos @323eb01, com cenário GO 5.0.3;
PEN1 é o controle negativo de CKEditor 5.

- Não há o mesmo `frmEditor`: a tela publica
  `window.INFRA_EDITOR_CONFIG`. Campos interpretados pelo Todos:
  `sei.urlSalvar`, `sei.versao`, `initialData`,
  `rootsAttributes` (seção `principal`, `somenteLeitura`,
  `label`) e `infra.estilo.itens[].permitidoEm`.
- No exemplo de GO há oito seções e quatro graváveis; o corpo fica
  no meio das graváveis. Ordem, nomes, rótulos e estilos não são
  invariantes. Sem declaração de gravabilidade, preservar a seção.
- O plugin servido `editor/ck5/dist/sei.umd.js` anuncia o
  endpoint `controlador_rest.php` com operação
  `editor_salvar_conteudo`. Não reconstruir sua URL.

## POST de salvar e resposta: o Todos afirma, cassette não conferida

| Família | Pedido que o Todos modela | Resposta que o Todos interpreta |
| --- | --- | --- |
| SEI 4 | POST dos controles de `frmEditor`, todas as seções, versão e não ignorar nova versão; resposta no iframe | `OK N` indica N; `INFRA_VALIDACAO` recusa (inclusive conflito); `divInfraExcecao` sinaliza erro; resposta diversa não confirma efeito |
| SEI 5 | POST `application/json` com `secoesConteudo` (todas as seções), `ignorarNovaVersao: "N"` e `versao` | JSON HTTP 200 com `versao` declara nova versão; `erro` recusa; `confirmarNovaVersao` ou `urlComparacao` indica conflito; 5xx/resposta desconhecida não prova ausência de efeito |

O protocolo e as respostas são descritos em
[`editor5.py`](https://github.com/franklinbaldo/todos/blob/323eb01/src/todos/sei/editor5.py)
e `_versao_salva`/`_versao_salva_5` em `documentos.py`.
O [catálogo](../cassettes.md) adverte que os corpos de POST
históricos não são completos: não usar os cards para certificar payload,
resposta ou sucesso sem uma leitura posterior.

## Nova versão e lista de versões

A barra publica `documento_versao_listar`. O
[contrato de leitura de versões](../../contracts/cli/documento-obter-versoes.md)
consulta N atual pelo rodapé ou pela lista e segue o link publicado
de versão histórica. No Todos,
`test_salvar_cria_proxima_versao_sem_reescrever_antiga`
simula 4 → 5, com verificação por leitura; é um **teste do oráculo**,
não demonstra por si só que a versão N+1 surgiu nos sete HARs.
A validação futura deve comparar: resposta do POST, rodapé,
lista de versões e conteúdo relido independentemente, rejeitando
apenas sucesso aparente.

## Documento assinado e impacto de editar

A barra `editarConteudo('S')` indica risco de edição de documento
assinado; a [caneta de assinatura](assinatura-do-documento.md)
é outra observação da árvore. Segundo o Todos, a página do documento
publica `documento_verificar_assinatura` (GET) e sua resposta
`SinAssinado` precisa ser `S` ou `N` para **aquele documento**.
Resposta ausente/inconclusiva recusa abrir o editor.

O Todos afirma que **abrir o editor pode remover assinaturas**
antes de qualquer POST de salvar. Seu teste
`test_assinado_nao_abre_editor_nem_grava_versao`
exige ausência de `editor_montar` em documento assinado.
Não reproduzir esse efeito num documento assinado para testar a
hipótese. O IES deve recusar antes da abertura e nunca presumir
que cancelar o editor restaure uma assinatura removida.

## Diferenças por instalação

| Instalação | Operação de salvar no card | Observação adicional nesta rodada |
| --- | --- | --- |
| UnB 4.0.9 | `editor_salvar` | Fixture do Todos mostra CKEditor 4 |
| UFRJ 4.1.5 | `editor_salvar` | Estrutura detalhada do HAR não conferida |
| PEN1 e PEN2 | `editor_salvar` | PEN1 é controle de CKEditor 4 no teste do Todos |
| GO 5.0.3 | `editor_salvar_conteudo` | Testes do Todos modelam CKEditor 5 |
| PEN3 e PEN4 5.0.4 | `editor_salvar_conteudo` | Estrutura detalhada do HAR não conferida |

As famílias decorrem dos **nomes de operações** nos cards e dos
testes do Todos. Não se deve generalizar campos obrigatórios,
semântica de assinatura ou status HTTP entre instalações.

## Perguntas que só ensaio ao vivo autorizado resolve

1. Abrir `editor_montar` de **documento não assinado** produz
   efeito mesmo sem salvar? Quais são os custos de GET?
2. Qual é o POST real por instalação/versão: campos obrigatórios,
   cabeçalhos, ordem de seções, resposta e status de aceitação/recusa?
3. A lista de versões, o rodapé e o HTML relido concordam sobre N+1?
   Há atraso, cache ou diferença na versão publicada?
4. Como distinguir conflito concorrente, recusa definitiva e
   escrita não confirmada após timeout, 5xx ou resposta desconhecida?
5. A assinatura é removida na abertura ou no salvar? E com
   múltiplos assinantes? **Não executar em assinado sem roteiro
   específico autorizado.**
6. Como são recusados documento externo, imagem, histórico,
   seção somente leitura e tela sem editor publicado?
7. Uma leitura independente consegue comprovar edição idempotente
   quando o conteúdo antes/depois é igual?

**Próxima prova:** obter os sete membros JSON dos ZIPs por seus
endereços registrados nos cards, verificar SHA-1 e SHA-256 normalizado,
inspecionar campos/scripts/respostas sem expor dados de sessão e
só então formalizar contrato e testes RED. O desenho das cópias
versionadas é o já decidido na [#207](https://github.com/franklinbaldo/ies/issues/207):
nenhum novo padrão de nomes é proposto aqui.
