#include <stdio.h>
#include <stdlib.h>

int main();
void game();
void shuffle(int *array, size_t n);
char* naipeChar(int num);
char* naipeValue(int num);
int cardValue(int card, int roundSuit, int trump);

int main()
{
    game();
    return 0;    
}

void game()
{
    int num_cards = 40;
    int num_hand = 10;
    int num_players = 4;
    int num_cards_per_suit = 10;
    int cards[num_cards];
    int playedCards[num_cards];
    /*
    Casa das unidades - Valor
    0 1 2 3 4 5 6 7 8 9
    2 3 4 5 6 Q J K 7 A
    
    Casa das dezenas - Naipe
    0 Copas
    1 Ouros
    2 Paus
    3 Espadas
    */
    for(int i = 0; i < num_cards; i++)
    {
        cards[i] = i;
        playedCards[i] = 0;
    }
    shuffle(cards, num_cards);

    int hand[num_players][num_hand];
    int trump_card = cards[0];
    int trump_suit = trump_card % num_cards_per_suit;
    for(int i = 0; i < num_players; i++)
    {
        for(int j = 0; j < num_hand; j++)
        {
            hand[i][j] = cards[i * num_hand + j];
            printf("%s%s ", naipeValue(hand[i][j]), naipeChar(hand[i][j]));
        }
        printf("\n");
    }
    // 10 games
    int initialPlayer = 0;
    for(int gameIndex = 0; gameIndex < 10; gameIndex++)
    {
        int roundCards[4] = { -1, -1, -1, -1 };
        int playerCardValue;
        int maxCardValue;
        int winner = initialPlayer;
        int roundSuit;
        int currentPlayer = initialPlayer;
        roundCards[currentPlayer] = nextCard(roundCards, hand[currentPlayer], playedCards);
        maxCardValue = (roundCards[currentPlayer] % num_cards_per_suit) + (roundCards[currentPlayer] / num_cards_per_suit == trump_suit ? 64 : 0);
        playedCards[roundCards[currentPlayer]] = 1; 
        roundSuit = roundCards[currentPlayer] / num_cards_per_suit;

        currentPlayer = (initialPlayer + 1) % num_players;
        roundCards[currentPlayer] = nextCard(roundCards, hand[currentPlayer], playedCards);
        playerCardValue = cardValue(roundCards[currentPlayer], roundSuit, trump_card);
        maxCardValue = playerCardValue > maxCardValue ? playerCardValue : maxCardValue;
        playedCards[roundCards[currentPlayer]] = 1;

        currentPlayer = (initialPlayer + 2) % num_players;
        roundCards[currentPlayer] = nextCard(roundCards, hand[currentPlayer], playedCards);
        playerCardValue = cardValue(roundCards[currentPlayer], roundSuit, trump_card);
        maxCardValue = playerCardValue > maxCardValue ? playerCardValue : maxCardValue;
        playedCards[roundCards[currentPlayer]] = 1;

        currentPlayer = (initialPlayer + 3) % num_players;
        roundCards[currentPlayer] = nextCard(roundCards, hand[currentPlayer], playedCards);
        playerCardValue = cardValue(roundCards[currentPlayer], roundSuit, trump_card);
        maxCardValue = playerCardValue > maxCardValue ? playerCardValue : maxCardValue;
        playedCards[roundCards[currentPlayer]] = 1;
    }
}

int nextCard(int roundCards[4], int hand[10], int playedCards[40])
{
    return hand[0];
}

int cardValue(int card, int roundSuit, int trump)
{
    int num_cards_per_suit = 10;
    int trump_value = 64;
    int cardSuit = card / 10;
    return cardSuit == roundSuit ? card % num_cards_per_suit + (cardSuit == trump ? trump_value : 0) : -1;
}

void shuffle(int *array, size_t n)
{
    if (n > 1) 
    {
        size_t i;
        for (i = 0; i < n - 1; i++) 
        {
          size_t j = i + rand() / (RAND_MAX / (n - i) + 1);
          int t = array[j];
          array[j] = array[i];
          array[i] = t;
        }
    }
}

char* naipeChar(int num)
{
    switch (num / 10)
    {
    case 0:
        return "♥";
        break;
    
    case 1:
        return "♦";
        break;

    case 2:
        return "♣";
        break;

    case 3:
        return "♠";
        break;

    default:
        return "\0";
        break;
    }
}

char* naipeValue(int num)
{
    switch (num % 10)
    {
    case 0:
        return "2";
        break;
    
    case 1:
        return "3";
        break;

    case 2:
        return "4";
        break;

    case 3:
        return "5";
        break;
    
    case 4:
        return "6";
        break;

    case 5:
        return "Q";
        break;

    case 6:
        return "J";
        break;

    case 7:
        return "K";
        break;

    case 8:
        return "7";
        break;

    case 9:
        return "A";
        break;

    default:
        return "\0";
        break;
    }
}