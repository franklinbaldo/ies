---
type: SEIObservation
title: Inclusão de documento interno pelo cadastro Web
description: Evidência preliminar da sequência de escolha de tipo, cadastro e conferência
---

# Inclusão de documento interno (issue #246)

## Fontes e limites

Oráculo comportamental: `franklinbaldo/todos/src/todos/documentos.py`,
funções `criar`, `_criar_na_arvore`, `_tipos_da_tela`,
`_escolha_do_tipo`, `cadastrar_da_tela` e `_depois_de_enviado`.
O acervo IES registra as cassettes
`cassettes/go/documento-incluir-go--fa4845e16ba9.md` (42 interações,
29 GET e 13 POST) e `cassettes/go/documento-escritas-go--16f83179ecaa.md`
(69 interações, 51 GET e 18 POST), ambas Goiás. A página capturada
`franklinbaldo/todos/tests/paginas_reais/documento_gerar.html` é
SEI 4.0.9. Esta nota é uma leitura das fontes: não houve replay do ZIP
nem consulta ao vivo nesta etapa.

## Caminho observado nas fontes

1. A árvore do processo fornece a ação de barra
   `documento_escolher_tipo` ("Incluir Documento"). O Todos a abre
   usando o link da barra; não sintetiza URL assinada.
2. A seleção usa a tabela `table#tblSeries`. O nome do tipo vem do texto
   do link. Nas formas descritas pelo oráculo, SEI 4.0 publica link para
   `documento_gerar`; SEI 4.1/5.0 publica `escolher(<id>)`.
   A verificação do Todos `tests/test_escolha_na_lista_010.py` @323eb01
   usa cassette da UFRJ (4.1.5): `escolher(291)` grava `hdnIdSerie`
   e submete `frmDocumentoEscolherTipo`, gerando POST de navegação,
   302 para `documento_gerar` e GET do cadastro. Isso não cria documento.
   A função é lida como estrutura literal e jamais executada.
   O item "Externo" não representa documento interno. Nome repetido
   impede escolha segura. A lista pode ter "Exibir todos", que usa
   `hdnFiltroSerie=T` em um POST: esse POST é de filtro, não de inclusão,
   mas também deve ter efeito avaliado e orçamento explícito.
3. Abrir o tipo leva ao cadastro `documento_gerar`. A captura mostra
   o título do tipo em `label#lblSerieTitulo`, o campo
   `txtDescricao`, as opções `rdoNivelAcesso` e, conforme o nível,
   `selHipoteseLegal`. Os campos reais precisam ser derivados do
   formulário servido. A presença de elementos ocultos não autoriza
   preencher valores não mostrados ao usuário.
4. No oráculo, `cadastrar_da_tela` confere o cadastro antes do envio;
   `enviar_formulario` executa uma submissão. O retorno é classificado
   e sua localização pode trazer `id_documento`. Essa informação
   isolada **não** confirma número SEI nem unicidade da inclusão.
5. `_depois_de_enviado` faz verificação posterior da árvore. Para o IES,
   a confirmação deve comparar os documentos de antes/depois,
   exigir exatamente um novo nó do tipo solicitado e devolver o número
   SEI publicado pela árvore. Falha de envio ou releitura com efeito
   não demonstrável deve resultar em saída 1, sem retry.

## Orçamento da navegação 4.1/5.0

Depois da leitura da árvore e da tela de seleção, uma escolha `escolher(id)` comprovada acrescenta **1 POST de navegação** (não de criação) e **1 GET do cadastro**; no 4.0 o link direto precisa apenas de **1 GET do cadastro**. A confirmação acrescenta **1 POST de criação**, nunca repetido, e uma leitura nova da árvore/processo. O número total da abertura depende dos atalhos observados no índice, não podendo ser declarado constante sem teste integrado. A derivação do corpo do POST de escolha é testada com fixture sintética e sem chamadas de rede; o replay e os totais live continuam NOT_RUN.

## Restrições para implementação

A prévia só deve abrir telas avaliadas como leitura; em particular,
`documento_gerar_multiplo` não deve ser invocado como escrita.
Antes do POST, a unidade da sessão e a situação aberta do processo
devem ser conferidas; tipos ausentes e ambíguos recusados por
subcódigos distintos. URLs assinadas, cookie e `infra_hash` não são
publicados. A solicitação de criação ao vivo permanece **NOT_RUN** até
a frase "escrita autorizada" na issue #246.

## Lacunas que impedem afirmar GREEN

Faltam replay dos corpos e respostas brutas das cassettes, teste RED,
implementação Rust da CLI/formulário e compilação pelo gate. As
capturas não permitem presumir que o mesmo formulário funciona em
todas as instalações. Nenhuma escrita foi realizada.

## Forma literal da função de gravação (inspeção de 05/11/2026)

No arquivo do oráculo
`Todos tests/paginas_reais/documento_gerar.html @323eb01`,
linhas 622–633, a página SEI 4.0.9 publica
`confirmarDados()` e `function submeter(){…}`.
O corpo direto de `submeter` executaria, nesta ordem, a desabilitação do
botão Salvar, a atribuição literal
`document.getElementById('hdnFlagDocumentoCadastro').value = '2';`
e `document.getElementById('frmDocumentoCadastro').submit();`.
Isso é **observação estática do HTML capturado**, não execução de JS nem
prova de criação ao vivo. **Nem mesmo uma atribuição isolada de
`hdnFlagDocumentoCadastro`** demonstra o caminho, pois falta a chamada
de submissão. Instruções em `if (false)` ou função arbitrária também
são insuficientes. O IES exige a função completa observada ou falha
antes do POST.


## Vínculo do botão Salvar com a confirmação (06/11/2026)

Na captura Todos `tests/paginas_reais/documento_gerar.html @323eb01`,
linhas 810 e 1061, existem dois botões `button#btnSalvar`, ambos
`type="button"` e `onclick="confirmarDados()"`. A função
`confirmarDados()` (linhas 615–619) chama `submeter()` somente depois
que `OnSubmitForm()` aprova os dados. Uma função `submeter()` presente
em um `<script>`, mas sem controle Salvar que a acione, é **código
potencialmente inativo** e não comprova a oferta de criação pela tela.
Antes de permitir o POST, exigir ao menos um botão Salvar habilitado com
o acionador literal observado, e recusar a tela inteira quando houver
outro `button#btnSalvar` divergente. Os dois botões iguais são
legítimos na captura; não exigir ID único nesse caso. Nunca executar JS.
A conferência desses atributos usa `html_atributos` e não substitui a
validação de controles obrigatórios feita no formulário.

## Cadeia completa de Salvar (06/11/2026)

Na captura Todos `documento_gerar.html @323eb01` (linhas 615–633),
`confirmarDados()` chama `submeter()` somente após
`if (OnSubmitForm())`. A função `submeter()` desabilita Salvar,
reescreve a flag e envia `frmDocumentoCadastro`. O botão com
`onclick="confirmarDados()"` é apenas a primeira ligação; uma função
`submeter()` morta não comprova a oferta de criação. A guarda exige
a cadeia literal completa, sem executar JavaScript, e recusa caminhos
ausentes, desviados ou duplicados antes do POST.


## Link de tipo com ID repetido no oráculo (06/11/2026)

No oráculo Todos `tests/paginas_reais/documento_escolher_tipo.html`
@323eb01, o `href` de **Despacho** contém **duas ocorrências
idênticas** de `id_procedimento=50693`, antes e depois de
`id_serie=291`. A URL é publicada como link de navegação do tipo;
não é inventada pelo cliente. Uma segunda ocorrência divergente NÃO
comprova identidade e deve ser recusada. Não registrar a URL assinada
nem seu valor `infra_hash` em fixtures ou logs.

Dois ensaios de treinamento UnB, usando o binário do head da PR #248,
confirmaram login em **5 requisições** cada e falharam fechados:
- Processo `23106.001875/2026-93`: prévia `FAIL`,
  `processo_nao_aberto_na_unidade_da_sessao`, sem POST criador.
- Processo distinto `23106.001868/2026-91`: prévia `FAIL`,
  `endereco_do_tipo_de_documento_nao_reconhecido`, sem POST criador.

Execuções: [primeira](https://github.com/franklinbaldo/ies/actions/runs/42183949372)
e [segunda](https://github.com/franklinbaldo/ies/actions/runs/42184286157).
O link efetivo da segunda tela não foi capturado; a duplicação do
oráculo é uma **hipótese verificável** para a segunda recusa, não uma
causa já comprovada ao vivo. Nenhum `--confirmar` foi executado,
nenhum documento foi criado. Testar a aceitação restrita de IDs
idênticos **sem alterar nem reconstruir** a URL publicada.


## Conteúdo inerte em template (verificação de segurança, 06/11/2026)

A [semântica HTML de `template`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/template) distingue o fragmento armazenado do DOM ativo: conteúdo aninhado, incluindo botões e scripts, não representa por si só controle acionável da tela. A captura positiva Todos @323eb01 já forneceu a cadeia Salvar → `confirmarDados` → `submeter`; um texto idêntico dentro de `template` não acrescenta evidência de execução. Este é um caso **adversarial sintético**, não uma forma observada ao vivo na UnB.

O leitor deve recusar a confirmação **antes de qualquer POST** se a única cadeia ou o único botão estiverem dentro de `template`, sem considerar o conteúdo inerte como prova de oferta de escrita. A prévia permanece somente leitura. A divergência live `submissao_do_documento_nao_observada` do [ensaio 42198707929](https://github.com/franklinbaldo/ies/actions/runs/42198707929) continua sem diagnóstico estrutural; não presumir que se trate de `template` nem relaxar a guarda com base nessa hipótese.
